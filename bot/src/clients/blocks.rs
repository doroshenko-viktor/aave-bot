use alloy::{
    eips::BlockNumberOrTag,
    providers::{Provider, RootProvider},
    pubsub::PubSubFrontend,
    rpc::types::{Block, BlockTransactionsKind},
};
use eyre::Result;
use futures::future::join_all;

use crate::PubSubNodeProvider;

pub async fn get_current_block(provider: &PubSubNodeProvider) -> Result<u64> {
    let x = provider.get_block_number().await?;
    Ok(x)
}

pub async fn get_blocks(
    provider: &RootProvider<PubSubFrontend>,
    block_numbers: &[u64],
) -> Vec<Block> {
    let fetch_futures = block_numbers.iter().map(|&block_number| async move {
        provider
            .get_block_by_number(
                BlockNumberOrTag::Number(block_number),
                BlockTransactionsKind::Full,
            )
            .await
    });

    join_all(fetch_futures)
        .await
        .into_iter()
        .flatten()
        .flatten()
        .collect::<Vec<Block>>()
}

pub async fn get_block(provider: &RootProvider<PubSubFrontend>, block_number: u64) -> Block {
    provider
        .get_block(block_number.into(), BlockTransactionsKind::Full)
        .await
        .unwrap()
        .unwrap()
}
