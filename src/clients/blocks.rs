use std::num::NonZero;

use alloy::{eips::BlockNumberOrTag, providers::Provider, rpc::types::Block};
use eyre::{Result, eyre};
use futures::future::join_all;
use tracing::error;

use crate::{NodeWsProvider, configuration::globals};

/*
Returns latest existing block number in blockchain
*/
pub async fn get_current_block(provider: &NodeWsProvider) -> Result<u64> {
    globals::RPC_RATE_LIMITER
        .until_n_ready(NonZero::new(5).unwrap())
        .await?;
    let result = match provider.get_block_number().await {
        Ok(val) => val,
        Err(err) => {
            match err {
                alloy::transports::RpcError::ErrorResp(error_payload) => {
                    todo!()
                }
                alloy::transports::RpcError::NullResp => todo!(),
                alloy::transports::RpcError::UnsupportedFeature(_) => todo!(),
                alloy::transports::RpcError::LocalUsageError(error) => todo!(),
                alloy::transports::RpcError::SerError(error) => todo!(),
                alloy::transports::RpcError::DeserError { err, text } => {
                    todo!()
                }
                alloy::transports::RpcError::Transport(e) => error!("{}", e),
            }
            todo!();
        }
    };

    Ok(result)
}

pub async fn get_blocks(
    provider: &NodeWsProvider,
    block_numbers: &[u64],
) -> Vec<Block> {
    let fetch_futures = block_numbers.iter().map(|&block_number| async move {
        provider
            .get_block_by_number(BlockNumberOrTag::Number(block_number))
            .await
    });

    join_all(fetch_futures)
        .await
        .into_iter()
        .flatten()
        .flatten()
        .collect::<Vec<Block>>()
}

pub async fn get_block(provider: &NodeWsProvider, block_number: u64) -> Block {
    provider
        .get_block(block_number.into())
        .await
        .unwrap()
        .unwrap()
}
