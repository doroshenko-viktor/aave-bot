use super::pool_contract::AavePoolContract;
use crate::{
    NodeHttpProvider,
    accounts::types::AssetAmount,
    blockchain::errors::BlockchainContractCallError,
    configuration::globals::{self, get_global_settings},
    tokens::TokenAddress,
};
use alloy::{
    primitives::TxHash, providers::Provider, rpc::types::TransactionReceipt,
};
use eyre::{Context, OptionExt, Result};
use governor::{InsufficientCapacity, Jitter};
use std::{
    num::NonZero,
    sync::{Arc, LazyLock},
    time::Duration,
};
use tokio::sync::RwLock;
use tracing::{error, info, warn};

const SUCCESS_RESET_COUNT: u32 = 10;
const FAILURE_RESET_COUNT: u32 = 50;
static CREDIT_RATIO: LazyLock<Arc<RwLock<(f64, u32)>>> =
    LazyLock::new(|| Arc::new(RwLock::new((1.0, SUCCESS_RESET_COUNT))));
const TRANSACTION_CREDITS: u32 = 10;
const VALUATION_CREDITS: u32 = 50;

pub trait BlockchainDataProviderFacade: Send + Sync {
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
}

pub struct BlockchainDataProviderFacadeImpl<T>
where
    T: AavePoolContract,
{
    pub provider: NodeHttpProvider,
    pub pool_contract: T,
}

impl<T> BlockchainDataProviderFacadeImpl<T>
where
    T: AavePoolContract + Send + Sync,
{
    pub fn new(provider: NodeHttpProvider, pool_contract: T) -> Self {
        Self { provider, pool_contract }
    }
}

impl<T> BlockchainDataProviderFacade for BlockchainDataProviderFacadeImpl<T>
where
    T: AavePoolContract + Send + Sync,
{
    async fn get_native_token_valuation(
        &self,
        a_token_address: TokenAddress,
        a_token_amount: AssetAmount,
        block_number: u64,
    ) -> Result<AssetAmount> {
        info!("Retrieving native token valuation");
        let callable = async || {
            self.pool_contract
                .get_native_token_valuation(
                    a_token_address,
                    a_token_amount,
                    block_number,
                )
                .await
        };
        with_retry(callable, VALUATION_CREDITS).await
    }

    async fn get_reserve_token_valuation(
        &self,
        token_address: TokenAddress,
        native_amount: AssetAmount,
        block_number: u64,
    ) -> Result<AssetAmount> {
        info!("Retrieving a-token valuation");
        let callable = async || {
            self.pool_contract
                .get_a_token_valuation(
                    token_address,
                    native_amount,
                    block_number,
                )
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

    async fn get_scaled_borrow_balance(
        &self,
        token_address: TokenAddress,
        native_amount: AssetAmount,
        block_number: u64,
    ) -> Result<AssetAmount> {
        let callable = async || {
            self.pool_contract
                .get_scaled_borrow_balance(
                    token_address,
                    native_amount,
                    block_number,
                )
                .await
        };

        let x = with_retry(callable, VALUATION_CREDITS)
            .await
            .context("Failed to get scaled borrow balance")?;

        Ok(x)
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
