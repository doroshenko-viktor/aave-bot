use alloy::{eips::BlockNumberOrTag, providers::Provider, rpc::types::Block};
use eyre::Result;
use futures::future::join_all;

use crate::NodeWsProvider;

/*
Returns latest existing block number in blockchain
*/
pub async fn get_current_block(provider: &NodeWsProvider) -> Result<u64> {
    Ok(provider.get_block_number().await?)
}

pub async fn get_blocks(provider: &NodeWsProvider, block_numbers: &[u64]) -> Vec<Block> {
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
