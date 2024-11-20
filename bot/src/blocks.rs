use alloy::{
    eips::BlockNumberOrTag,
    providers::{Provider, RootProvider},
    pubsub::PubSubFrontend,
    rpc::types::{Block, BlockTransactionsKind},
};
use futures::future::join_all;

pub async fn get_blocks(
    provider: &RootProvider<PubSubFrontend>,
    block_numbers: &[u64],
) -> Vec<Block> {
    let fetch_futures = block_numbers.iter().map(|&block_number| async move {
        provider
            .get_block_by_number(BlockNumberOrTag::Number(block_number), true)
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
    let block = provider
        .get_block(block_number.into(), BlockTransactionsKind::Full)
        .await
        .unwrap()
        .unwrap();
    println!("Gas limit: {:?}", block.header.gas_limit);
    println!("Gas used: {:?}", block.header.gas_used);
    println!("Miner: {:?}", block.header.miner);
    println!("Difficulty: {:?}", block.header.difficulty);
    println!("Block size: {:?}", block.size);
    let transaction = block
        .transactions
        .as_transactions()
        .unwrap()
        .first()
        .unwrap();
    println!("Transaction: {:#?}", transaction);
    println!("{}", block_number);

    block
}
