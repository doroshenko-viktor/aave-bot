use super::{blocks::BlockNumber, pool_contract::AavePoolContract};
use crate::{
    NodeHttpProvider,
    accounts::types::AssetAmount,
    blockchain::errors::BlockchainContractCallError,
    configuration::globals::{self, get_global_settings},
    tokens::TokenAddress,
};
use alloy::{
    primitives::{TxHash, U256, Uint},
    providers::Provider,
    rpc::{
        client::{RpcClient, Waiter},
        types::{Transaction, TransactionReceipt},
    },
};
use dashmap::DashMap;
use eyre::{Context, OptionExt, Result};
use futures::future::join_all;
use governor::Jitter;
use hashbrown::HashSet;
use reqwest::Client;
use serde_json::{Value, json};
use std::{
    num::NonZero,
    ops::Add,
    sync::{Arc, LazyLock},
    time::Duration,
};
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

const RAY: u128 = 1_000_000_000_000_000_000_000_000_000; // 1e27
const SUCCESS_RESET_COUNT: u32 = 10;
const FAILURE_RESET_COUNT: u32 = 50;
static CREDIT_RATIO: LazyLock<Arc<RwLock<(f64, u32)>>> =
    LazyLock::new(|| Arc::new(RwLock::new((1.0, SUCCESS_RESET_COUNT))));
const TRANSACTION_CREDITS: u32 = 10;
const VALUATION_CREDITS: u32 = 50;

pub trait BlockchainDataProviderFacade: Send + Sync + Clone {
    fn prefetch_transactions(
        &self,
        transaction_hashes: HashSet<TxHash>,
    ) -> impl Future<Output = Result<()>>;

    fn prefetch_normalized_income(
        &self,
        collateral_tokens: HashSet<(u64, Vec<TokenAddress>)>,
        debt_tokens: HashSet<(u64, Vec<TokenAddress>)>,
    ) -> impl Future<Output = Result<()>>;

    fn get_native_token_valuation(
        &self,
        token_address: TokenAddress,
        a_token_amount: AssetAmount,
        block_number: u64,
    ) -> impl Future<Output = Result<AssetAmount>>;

    fn get_reserve_token_valuation(
        &self,
        token_address: TokenAddress,
        native_amount: AssetAmount,
        block_number: u64,
    ) -> impl Future<Output = Result<AssetAmount>>;

    fn get_scaled_borrow_balance(
        &self,
        token_address: TokenAddress,
        native_amount: AssetAmount,
        block_number: u64,
    ) -> impl Future<Output = Result<AssetAmount>>;

    /**
     * Get transaction receipt by hash
     */
    fn get_transaction(
        &self,
        transaction_hash: TxHash,
    ) -> impl Future<Output = Result<TransactionReceipt>>;

    async fn get_transactions_batch(
        &self,
        transaction_hashes: Vec<TxHash>,
    ) -> Result<Vec<(TxHash, Transaction)>, BlockchainContractCallError>;

    fn get_reserve_token_normalized_income(
        &self,
        token_address: TokenAddress,
        block_number: u64,
    ) -> impl Future<Output = Result<Uint<256, 4>>>;

    fn get_debt_token_normalized_income(
        &self,
        token_address: TokenAddress,
        block_number: u64,
    ) -> impl Future<Output = Result<Uint<256, 4>>>;
}

#[derive(Debug, Clone)]
pub struct BlockchainDataProviderFacadeImpl<T>
where
    T: AavePoolContract + Clone + Send + Sync,
{
    provider: NodeHttpProvider,
    rpc_client: RpcClient,
    pool_contract: T,
    transaction_cache: Arc<DashMap<TxHash, Transaction>>,
    reserve_normalized_income_cache:
        Arc<DashMap<(BlockNumber, TokenAddress), Uint<256, 4>>>,
    normalized_variable_debt_cache:
        Arc<DashMap<(BlockNumber, TokenAddress), Uint<256, 4>>>,
}

impl<T> BlockchainDataProviderFacadeImpl<T>
where
    T: AavePoolContract + Send + Sync + Clone,
{
    pub fn new(
        provider: NodeHttpProvider,
        rpc_client: RpcClient,
        pool_contract: T,
    ) -> Self {
        Self {
            provider,
            pool_contract,
            rpc_client,
            transaction_cache: Arc::new(DashMap::new()),
            reserve_normalized_income_cache: Arc::new(DashMap::new()),
            normalized_variable_debt_cache: Arc::new(DashMap::new()),
        }
    }
}

impl<T> BlockchainDataProviderFacade for BlockchainDataProviderFacadeImpl<T>
where
    T: AavePoolContract + Send + Sync + Clone,
{
    async fn prefetch_transactions(
        &self,
        transaction_hashes: HashSet<TxHash>,
    ) -> Result<()> {
        warn!(
            "Prefetching transactions count: {:#?}",
            transaction_hashes.len()
        );

        self.transaction_cache.clear();

        let mut delay = Duration::from_millis(500);
        let mut batch_ind = 0;
        let mut chunk = Vec::with_capacity(10);
        for value in transaction_hashes {
            chunk.push(value);
            if chunk.len() < 10 {
                continue;
            }

            loop {
                let chunk = chunk.clone();
                // globals::RPC_RATE_LIMITER
                //     .until_n_ready_with_jitter(
                //         NonZero::try_from(200)?,
                //         Jitter::new(
                //             Duration::from_millis(10),
                //             Duration::from_millis(50),
                //         ),
                //     )
                //     .await?;
                tokio::time::sleep(delay).await;

                match self.get_transactions_batch(chunk).await {
                    Ok(res) => {
                        info!(
                            "retrieved batch #{} of transactions",
                            batch_ind
                        );
                        for (tx_hash, transaction) in res {
                            self.transaction_cache.insert(tx_hash, transaction);
                        }
                        batch_ind += 1;
                        break;
                    }
                    Err(BlockchainContractCallError::TooManyAttempts(_)) => {
                        warn!(
                            "waiting after too many attempts response from rpc node for {}",
                            delay.as_millis()
                        );
                        tokio::time::sleep(delay).await;
                        delay = delay.add(delay.mul_f32(0.2));
                        continue;
                    }
                    Err(BlockchainContractCallError::Other(e)) => {
                        error!(
                            "Error getting transactions batch: {:#?}",
                            e
                        );
                        return Err(e);
                    }
                };
            }
        }

        Ok(())
    }

    async fn prefetch_normalized_income(
        &self,
        collateral_tokens: HashSet<(u64, Vec<TokenAddress>)>,
        debt_tokens: HashSet<(u64, Vec<TokenAddress>)>,
    ) -> Result<()> {
        self.reserve_normalized_income_cache.clear();
        self.normalized_variable_debt_cache.clear();

        let mut chunk = Vec::with_capacity(50);
        for value in collateral_tokens {
            chunk.push(value);
            if chunk.len() < 50 {
                continue;
            }

            loop {
                let chunk = chunk.clone();
                globals::RPC_RATE_LIMITER
                    .until_n_ready_with_jitter(
                        NonZero::try_from(499)?,
                        Jitter::new(
                            Duration::from_millis(10),
                            Duration::from_millis(50),
                        ),
                    )
                    .await?;
                match self
                    .pool_contract
                    .get_normalized_income_batch(chunk)
                    .await
                {
                    Ok(res) => {
                        for (block_number, token_address, normalized_income) in
                            res
                        {
                            self.reserve_normalized_income_cache.insert(
                                (block_number, token_address),
                                normalized_income,
                            );
                        }
                        break;
                    }
                    Err(BlockchainContractCallError::TooManyAttempts(_)) => {
                        tokio::time::sleep(Duration::from_secs(1)).await;
                        continue;
                    }
                    Err(BlockchainContractCallError::Other(e)) => {
                        error!(
                            "Error getting normalized income: {:#?}",
                            e
                        );
                        return Err(e);
                    }
                };
            }
        }

        for value in debt_tokens {
            chunk.push(value);
            if chunk.len() < 50 {
                continue;
            }

            loop {
                let chunk = chunk.clone();
                globals::RPC_RATE_LIMITER
                    .until_n_ready_with_jitter(
                        NonZero::try_from(499)?,
                        Jitter::new(
                            Duration::from_millis(10),
                            Duration::from_millis(50),
                        ),
                    )
                    .await?;
                match self
                    .pool_contract
                    .get_variable_debt_normalized_income_batch(chunk)
                    .await
                {
                    Ok(res) => {
                        for (block_number, token_address, normalized_income) in
                            res
                        {
                            self.reserve_normalized_income_cache.insert(
                                (block_number, token_address),
                                normalized_income,
                            );
                        }
                        break;
                    }
                    Err(BlockchainContractCallError::TooManyAttempts(_)) => {
                        tokio::time::sleep(Duration::from_secs(1)).await;
                        continue;
                    }
                    Err(BlockchainContractCallError::Other(e)) => {
                        error!(
                            "Error getting normalized income: {:#?}",
                            e
                        );
                        return Err(e);
                    }
                };
            }
        }

        Ok(())
    }

    async fn get_native_token_valuation(
        &self,
        a_token_address: TokenAddress,
        a_token_amount: AssetAmount,
        block_number: u64,
    ) -> Result<AssetAmount> {
        debug!("Retrieving native token valuation");
        let normalized_income = if let Some(cached) = self
            .reserve_normalized_income_cache
            .get(&(block_number, a_token_address))
        {
            cached.value().clone()
        } else {
            let normalized_income = self
                .get_reserve_token_normalized_income(
                    a_token_address,
                    block_number,
                )
                .await?;
            debug!(
                "get_a_token_valuation normalized_income: {:?}; token_address: {:#x}; block_number: {:?}",
                normalized_income, a_token_address, block_number
            );
            normalized_income
        };

        let ray = U256::from(RAY);

        Ok(a_token_amount
            .checked_mul(normalized_income)
            .unwrap()
            .checked_div(ray)
            .unwrap())
    }

    async fn get_reserve_token_valuation(
        &self,
        token_address: TokenAddress,
        native_amount: AssetAmount,
        block_number: u64,
    ) -> Result<AssetAmount> {
        info!("Retrieving a-token valuation");
        let normalized_income = if let Some(cached) = self
            .reserve_normalized_income_cache
            .get(&(block_number, token_address))
        {
            cached.value().clone()
        } else {
            let normalized_income = self
                .get_reserve_token_normalized_income(
                    token_address,
                    block_number,
                )
                .await?;
            debug!(
                "get_a_token_valuation normalized_income: {:?}; token_address: {:#x}; block_number: {:?}",
                normalized_income, token_address, block_number
            );
            normalized_income
        };

        let ray = U256::from(RAY);

        let result = native_amount
            .checked_mul(ray)
            .unwrap() // TODO: make error handling
            .checked_div(normalized_income)
            .unwrap();

        Ok(result)
    }

    async fn get_scaled_borrow_balance(
        &self,
        token_address: TokenAddress,
        native_amount: AssetAmount,
        block_number: u64,
    ) -> Result<AssetAmount> {
        let normalized_income = if let Some(normalized_income) = self
            .normalized_variable_debt_cache
            .get(&(block_number, token_address))
        {
            *normalized_income.value()
        } else {
            self.get_debt_token_normalized_income(token_address, block_number)
                .await?
        };

        let ray = U256::from(RAY);
        let result = native_amount
            .checked_mul(ray)
            .unwrap()
            .checked_div(normalized_income)
            .unwrap();

        Ok(result)
    }

    async fn get_reserve_token_normalized_income(
        &self,
        token_address: TokenAddress,
        block_number: u64,
    ) -> Result<Uint<256, 4>> {
        let callable = async || {
            self.pool_contract
                .get_reserve_token_normalized_income(
                    token_address,
                    block_number,
                )
                .await
        };

        with_retry(callable, VALUATION_CREDITS).await
    }

    async fn get_debt_token_normalized_income(
        &self,
        token_address: TokenAddress,
        block_number: u64,
    ) -> Result<Uint<256, 4>> {
        let callable = async || {
            self.pool_contract
                .get_debt_token_normalized_income(token_address, block_number)
                .await
        };

        with_retry(callable, VALUATION_CREDITS).await
    }

    async fn get_transaction(
        &self,
        transaction_hash: TxHash,
    ) -> Result<TransactionReceipt> {
        let callable = async || {
            self.provider
                .get_transaction_receipt(transaction_hash)
                .await
                .map_err(|err| err.into())
        };

        with_retry(callable, TRANSACTION_CREDITS)
            .await
            .context("Failed to get transaction receipt")?
            .ok_or_eyre("Transaction receipt not found")
    }

    async fn get_transactions_batch(
        &self,
        transaction_hashes: Vec<TxHash>,
    ) -> Result<Vec<(TxHash, Transaction)>, BlockchainContractCallError> {
        let (batch, requests) = transaction_hashes.into_iter().fold(
            (
                self.rpc_client.new_batch(),
                Vec::<_>::new(),
            ),
            |mut acc, tx_hash| {
                let result_fut = acc.0.add_call::<_, Option<Transaction>>(
                    "eth_getTransactionByHash",
                    &[tx_hash],
                );

                acc.1.push((result_fut, tx_hash));
                acc
            },
        );

        if let Err(err) = batch.send().await {
            error!(
                "error sending transactions batch: {}",
                err
            );
            match err {
                alloy_json_rpc::RpcError::DeserError { err, text } => {
                    error!("DeserError: err: {}", err);
                    if text.contains("32005") {
                        return Err(
                            BlockchainContractCallError::TooManyAttempts(
                                "".to_string(),
                            ),
                        );
                    }
                }
                _ => return Err(err.into()),
            }
        }

        let results = join_all(requests.into_iter().map(
            |(response, tx_hash)| async move {
                let result = match response {
                    Ok(value) => {
                        value.await.map_err(BlockchainContractCallError::from)
                    }
                    Err(e) => Err(e.into()),
                };
                // response.await.map_err(BlockchainContractCallError::from);
                (result, tx_hash)
            },
        ))
        .await;

        let mut acc = Vec::new();
        for (result, tx_hash) in results {
            match result {
                Ok(Some(result)) => acc.push((tx_hash, result)),
                Err(BlockchainContractCallError::TooManyAttempts(e)) => {
                    error!("too many attemts fetching batched transactions");
                    return Err(
                        BlockchainContractCallError::TooManyAttempts(e),
                    );
                }
                Err(err) => error!(
                    "error fetching transactions in batch: {}",
                    err
                ),
                _ => warn!(
                    "transaction for hash {} not found. Ignoring this due to cache prefetch nature of operation",
                    tx_hash
                ),
            };
        }

        Ok(acc)
    }
}

/**
 * Retry function with exponential backoff
 * `func` is a function that returns a result that can be retried
 * `base_credits` is the base number of credits to use for the delay. It will increase for each
 * next retry attemt until max node credit number configured globally
 */
async fn with_retry<T>(
    func: impl AsyncFn() -> Result<T, BlockchainContractCallError>,
    wait_credits: u32,
) -> std::result::Result<T, eyre::Error> {
    let max_blockchain_retry_count =
        get_global_settings().rpc_config.max_blockchain_retry_count;
    for i in 0..max_blockchain_retry_count {
        let current_credits = ((CREDIT_RATIO.read().await.0
            * wait_credits as f64) as u32)
            .min(get_global_settings().rpc_config.max_node_burst_size);
        info!(
            "Using {} credits for the {} attempt",
            current_credits, i
        );
        if current_credits > 0 {
            globals::RPC_RATE_LIMITER
                .until_n_ready_with_jitter(
                    NonZero::try_from(current_credits)?,
                    Jitter::new(
                        Duration::from_millis(10),
                        Duration::from_millis(50),
                    ),
                )
                .await?;
        }

        match func().await {
            Ok(value) => {
                if CREDIT_RATIO.read().await.1 > 1 {
                    CREDIT_RATIO.write().await.1 -= 1;
                    info!(
                        "Success response - reducing reset count {}",
                        CREDIT_RATIO.read().await.1
                    );
                } else {
                    let credits = {
                        let credit_ratio =
                            CREDIT_RATIO.read().await.0 as f64 * 0.9;
                        let reset_count = SUCCESS_RESET_COUNT;

                        (credit_ratio, reset_count)
                    };
                    *CREDIT_RATIO.write().await = credits;
                    info!(
                        "Success response - reducing wait credits: {:#?}",
                        CREDIT_RATIO.read().await
                    );
                }

                return Ok(value);
            }
            Err(BlockchainContractCallError::TooManyAttempts(body)) => {
                if i < max_blockchain_retry_count {
                    let credits = {
                        let multiplier = (2.0 + i as f64).ln();
                        let credit_ratio = CREDIT_RATIO.read().await.0
                            + CREDIT_RATIO.read().await.0 * multiplier;
                        let reset_count = FAILURE_RESET_COUNT;

                        (credit_ratio, reset_count)
                    };
                    *CREDIT_RATIO.write().await = credits;

                    warn!(
                        "FAILED: TOO MANY ATTEMPTS; Try #{}: {}; Retrying with increased credits: {:#?}",
                        i,
                        body,
                        CREDIT_RATIO.read().await
                    );
                } else {
                    error!(
                        "FAILED: TOO MANY ATTEMPTS: {}; No more retries",
                        body
                    );
                }
                continue;
            }
            Err(BlockchainContractCallError::Other(err)) => {
                return Err(err);
            }
        };
    }

    Err(eyre::eyre!(
        "Failed to get aToken valuation after {} retries",
        get_global_settings().rpc_config.max_blockchain_retry_count
    ))
}
