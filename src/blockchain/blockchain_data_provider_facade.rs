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
use std::num::NonZero;
use tracing::{error, info, warn};

pub trait BlockchainDataProviderFacade: Send + Sync {
    fn get_native_token_valuation(
        &self,
        token_address: TokenAddress,
        a_token_amount: AssetAmount,
        block_number: u64,
    ) -> impl Future<Output = Result<AssetAmount>>;

    fn get_a_token_valuation(
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
        token_address: TokenAddress,
        a_token_amount: AssetAmount,
        block_number: u64,
    ) -> Result<AssetAmount> {
        info!("Retrieving native token valuation");
        let callable = async || {
            self.pool_contract
                .get_native_token_valuation(
                    token_address,
                    a_token_amount,
                    block_number,
                )
                .await
        };
        with_retry(callable, 10).await
    }

    async fn get_a_token_valuation(
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
        with_retry(callable, 10).await
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

        with_retry(callable, 10)
            .await
            .context("Failed to get transaction receipt")?
            .ok_or_eyre("Transaction receipt not found")
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
    base_credits: u32,
) -> std::result::Result<T, eyre::Error> {
    let max_blockchain_retry_count =
        get_global_settings().rpc_config.max_blockchain_retry_count;
    let max_credits = get_global_settings().rpc_config.max_node_credits - 1;
    for i in 0..max_blockchain_retry_count {
        let multiplier = (1.0 + i as f64).ln();
        let credits = base_credits as f64 + base_credits as f64 * multiplier;
        let capped_credits = credits.min(max_credits as f64) as u32;

        warn!(
            "Trying to get blockchain data with {} of credits of maximum {}",
            capped_credits, max_credits
        );

        if capped_credits > 0 {
            globals::RPC_RATE_LIMITER
                .until_n_ready(NonZero::try_from(capped_credits)?)
                .await?;
        }

        match func().await {
            Ok(value) => return Ok(value),
            Err(BlockchainContractCallError::TooManyAttempts(body)) => {
                warn!("FAILED: TOO MANY ATTEMPTS: {}", body);
                if i < max_blockchain_retry_count {
                    warn!("Retrying...");
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
