use crate::{
    NodeHttpProvider, accounts::types::AssetAmount,
    contracts::pool::Pool::PoolInstance, tokens::TokenAddress,
};
use alloy::{
    eips::BlockId,
    primitives::{Address, U256, Uint},
    transports::{HttpError, RpcError, Transport, TransportError},
};
use eyre::{Context, Result};
use tracing::error;

use super::errors::BlockchainContractCallError;

const RAY: u128 = 1_000_000_000_000_000_000_000_000_000; // 1e27

pub trait AavePoolContract {
    /**
     * Get valuation of supplied asset in a-tokens
     */
    fn get_reserve_token_normalized_income(
        &self,
        account_address: TokenAddress,
        block_number: u64,
    ) -> impl Future<Output = Result<Uint<256, 4>, BlockchainContractCallError>>;

    fn get_native_token_valuation(
        &self,
        token_address: TokenAddress,
        a_token_amount: Uint<256, 4>,
        block_number: u64,
    ) -> impl Future<Output = Result<AssetAmount, BlockchainContractCallError>>;

    fn get_a_token_valuation(
        &self,
        token_address: TokenAddress,
        native_amount: Uint<256, 4>,
        block_number: u64,
    ) -> impl Future<Output = Result<AssetAmount, BlockchainContractCallError>>;
}

pub struct AavePoolContractBlockchainImpl {
    pool: PoolInstance<NodeHttpProvider>,
}

impl AavePoolContractBlockchainImpl {
    pub fn new(
        provider: NodeHttpProvider,
        pool_contract_address: Address,
    ) -> Self {
        let pool = PoolInstance::new(pool_contract_address, provider);
        Self { pool }
    }
}

impl AavePoolContract for AavePoolContractBlockchainImpl {
    async fn get_native_token_valuation(
        &self,
        token_address: TokenAddress,
        a_token_amount: Uint<256, 4>,
        block_number: u64,
    ) -> Result<Uint<256, 4>, BlockchainContractCallError> {
        let normalized_income = self
            .get_reserve_token_normalized_income(token_address, block_number)
            .await?;

        let ray = U256::from(RAY);

        Ok(a_token_amount
            .checked_mul(ray)
            .unwrap()
            .checked_div(normalized_income)
            .unwrap())
    }

    async fn get_a_token_valuation(
        &self,
        token_address: TokenAddress,
        native_amount: Uint<256, 4>,
        block_number: u64,
    ) -> Result<Uint<256, 4>, BlockchainContractCallError> {
        let normalized_income = self
            .get_reserve_token_normalized_income(token_address, block_number)
            .await?;

        let ray = U256::from(RAY);

        let result = native_amount
            .checked_mul(normalized_income)
            .unwrap() // TODO: make error handling
            .checked_div(ray)
            .unwrap();

        Ok(result)
    }

    async fn get_reserve_token_normalized_income(
        &self,
        token_address: TokenAddress,
        block_number: u64,
    ) -> Result<Uint<256, 4>, BlockchainContractCallError> {
        self.pool
            .getReserveNormalizedIncome(token_address)
            .block(BlockId::number(block_number))
            .call()
            .await
            .map_err(|err| err.into())
    }
}
