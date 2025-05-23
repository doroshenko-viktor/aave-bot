use std::sync::Arc;

use crate::{
    NodeHttpProvider, accounts::types::AssetAmount,
    contracts::pool::Pool::PoolInstance, tokens::TokenAddress,
};
use alloy::{
    eips::BlockId,
    hex::ToHexExt as _,
    primitives::{Address, U256, Uint},
    rpc::client::RpcClient,
};
use dashmap::DashMap;
use eyre::Result;
use futures::future::join_all;
use ringbuf::{
    HeapRb,
    traits::{Consumer, RingBuffer},
};
use serde_json::Value;
use tracing::{debug, error};

use super::{blocks::BlockNumber, errors::BlockchainContractCallError};

pub trait AavePoolContract {
    /**
     * Get valuation of supplied asset in a-tokens
     */
    fn get_reserve_token_normalized_income(
        &self,
        token_address: TokenAddress,
        block_number: u64,
    ) -> impl Future<Output = Result<Uint<256, 4>, BlockchainContractCallError>>;

    fn get_normalized_income_batch(
        &self,
        collateral_tokens: Vec<(BlockNumber, Vec<TokenAddress>)>,
    ) -> impl Future<
        Output = Result<
            Vec<(BlockNumber, TokenAddress, Uint<256, 4>)>,
            BlockchainContractCallError,
        >,
    >;

    fn get_variable_debt_normalized_income_batch(
        &self,
        debt_tokens: Vec<(BlockNumber, Vec<TokenAddress>)>,
    ) -> impl Future<
        Output = Result<
            Vec<(BlockNumber, TokenAddress, Uint<256, 4>)>,
            BlockchainContractCallError,
        >,
    >;

    /**
     * Get valuation of borrowed variable debt token
     */
    fn get_debt_token_normalized_income(
        &self,
        token_address: TokenAddress,
        block_number: u64,
    ) -> impl Future<Output = Result<Uint<256, 4>, BlockchainContractCallError>>;
}

#[derive(Clone)]
pub struct AavePoolContractBlockchainImpl {
    pool: PoolInstance<NodeHttpProvider>,
    rpc_client: RpcClient,
    reserve_normalized_income_cache:
        Arc<DashMap<TokenAddress, HeapRb<(u64, AssetAmount)>>>,
    debt_normalized_income_cache:
        Arc<DashMap<TokenAddress, HeapRb<(u64, AssetAmount)>>>,
}

impl AavePoolContractBlockchainImpl {
    pub fn new(
        provider: NodeHttpProvider,
        rpc_client: RpcClient,
        pool_contract_address: Address,
    ) -> Self {
        let pool = PoolInstance::new(pool_contract_address, provider);
        let reserve_normalized_income_cache = Arc::new(DashMap::new());
        let debt_normalized_income_cache = Arc::new(DashMap::new());
        Self {
            pool,
            rpc_client,
            reserve_normalized_income_cache,
            debt_normalized_income_cache,
        }
    }
}

impl AavePoolContract for AavePoolContractBlockchainImpl {
    async fn get_reserve_token_normalized_income(
        &self,
        token_address: TokenAddress,
        block_number: u64,
    ) -> Result<Uint<256, 4>, BlockchainContractCallError> {
        if let Some(cached) = self
            .reserve_normalized_income_cache
            .get(&token_address)
            .and_then(|x| x.iter().rfind(|x| x.0 == block_number).map(|x| x.1))
        {
            return Ok(cached);
        }

        let res = self
            .pool
            .getReserveNormalizedIncome(token_address)
            .block(BlockId::number(block_number))
            .call()
            .await
            .map_err(BlockchainContractCallError::from)?;
        let mut cache = self
            .reserve_normalized_income_cache
            .entry(token_address)
            .or_insert_with(|| HeapRb::new(5));
        cache.push_overwrite((block_number, res));

        Ok(res)
    }

    async fn get_normalized_income_batch(
        &self,
        tokens: Vec<(BlockNumber, Vec<TokenAddress>)>,
    ) -> Result<
        Vec<(BlockNumber, TokenAddress, Uint<256, 4>)>,
        BlockchainContractCallError,
    > {
        let (batch, requests) = tokens
            .into_iter()
            .flat_map(|(block_number, token_addresses)| {
                token_addresses
                    .into_iter()
                    .map(|token_address| {
                        (
                            block_number.clone(),
                            token_address.clone(),
                        )
                    })
                    .collect::<Vec<_>>()
            })
            .fold(
                (
                    self.rpc_client.new_batch(),
                    Vec::<_>::new(),
                ),
                |mut acc, (block_number, token_address)| {
                    let get_reserve_normalized_income_call = self
                        .pool
                        .getReserveNormalizedIncome(token_address)
                        .block(BlockId::number(block_number));
                    let call_data =
                        get_reserve_normalized_income_call.calldata();

                    let block_hex = format!("0x{:x}", block_number);
                    let params = serde_json::json!([
                        {
                            "to": self.pool.address(),
                            "data": format!("0x{}", call_data.encode_hex())
                        },
                        block_hex
                    ]);

                    let result_fut = acc
                        .0
                        .add_call::<Value, Uint<256, 4>>("eth_call", &params)
                        .unwrap();
                    acc.1.push((result_fut, token_address, block_number));
                    acc
                },
            );

        batch
            .send()
            .await
            .map_err(BlockchainContractCallError::from)?;
        let (results, errors) = join_all(requests.into_iter().map(
            |(response, token_address, block_number)| async move {
                let result =
                    response.await.map_err(BlockchainContractCallError::from);
                (result, token_address, block_number)
            },
        ))
        .await
        .into_iter()
        .fold(
            (Vec::<_>::new(), Vec::<_>::new()),
            |mut acc, (response, token_address, block_number)| {
                match response {
                    Ok(result) => {
                        acc.0.push((block_number, token_address, result))
                    }
                    Err(err) => acc.1.push(err),
                };
                acc
            },
        );

        if !errors.is_empty() {
            error!("Errors: {:#?}", errors);
            return Err(BlockchainContractCallError::Other(
                eyre::eyre!("Batch call normalized_income failed"),
            ));
        }

        Ok(results)
    }

    async fn get_debt_token_normalized_income(
        &self,
        token_address: TokenAddress,
        block_number: u64,
    ) -> Result<Uint<256, 4>, BlockchainContractCallError> {
        if let Some(cached) = self
            .debt_normalized_income_cache
            .get(&token_address)
            .and_then(|x| x.iter().rfind(|x| x.0 == block_number).map(|x| x.1))
        {
            return Ok(cached);
        }
        let res = self
            .pool
            .getReserveNormalizedVariableDebt(token_address)
            .block(BlockId::number(block_number))
            .call()
            .await
            .map_err(BlockchainContractCallError::from)?;

        let mut cache = self
            .debt_normalized_income_cache
            .entry(token_address)
            .or_insert_with(|| HeapRb::new(5));
        cache.push_overwrite((block_number, res));

        Ok(res)
    }

    async fn get_variable_debt_normalized_income_batch(
        &self,
        tokens: Vec<(BlockNumber, Vec<TokenAddress>)>,
    ) -> Result<
        Vec<(BlockNumber, TokenAddress, Uint<256, 4>)>,
        BlockchainContractCallError,
    > {
        let (batch, requests) = tokens
            .into_iter()
            .flat_map(|(block_number, token_addresses)| {
                token_addresses
                    .into_iter()
                    .map(|token_address| (block_number, token_address))
                    .collect::<Vec<_>>()
            })
            .fold(
                (
                    self.rpc_client.new_batch(),
                    Vec::<_>::new(),
                ),
                |mut acc, (block_number, token_address)| {
                    let get_reserve_normalized_income_call = self
                        .pool
                        .getReserveNormalizedVariableDebt(token_address)
                        .block(BlockId::number(block_number));
                    let call_data =
                        get_reserve_normalized_income_call.calldata();

                    let block_hex = format!("0x{:x}", block_number);
                    let params = serde_json::json!([
                        {
                            "to": self.pool.address(),
                            "data": format!("0x{}", call_data.encode_hex())
                        },
                        block_hex
                    ]);

                    let result_fut = acc
                        .0
                        .add_call::<Value, Uint<256, 4>>("eth_call", &params)
                        .unwrap();
                    acc.1.push((result_fut, token_address, block_number));
                    acc
                },
            );

        batch
            .send()
            .await
            .map_err(BlockchainContractCallError::from)?;
        let (results, errors) = join_all(requests.into_iter().map(
            |(response, token_address, block_number)| async move {
                let result =
                    response.await.map_err(BlockchainContractCallError::from);
                (result, token_address, block_number)
            },
        ))
        .await
        .into_iter()
        .fold(
            (Vec::<_>::new(), Vec::<_>::new()),
            |mut acc, (response, token_address, block_number)| {
                match response {
                    Ok(result) => {
                        acc.0.push((block_number, token_address, result))
                    }
                    Err(err) => acc.1.push(err),
                };
                acc
            },
        );

        if !errors.is_empty() {
            error!("Errors: {:#?}", errors);
            return Err(BlockchainContractCallError::Other(
                eyre::eyre!("Batch call normalized_income failed"),
            ));
        }

        Ok(results)
    }
}
