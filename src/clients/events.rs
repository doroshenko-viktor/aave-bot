use std::{collections::VecDeque, sync::Arc};

use alloy::{
    eips::BlockNumberOrTag,
    providers::{Provider, RootProvider},
    pubsub::PubSubFrontend,
    rpc::types::{Filter, Log},
    sol_types::SolEvent,
};
use eyre::{Error, Result};
use futures::lock::Mutex;
use tokio::sync::mpsc::Receiver;

use crate::{
    NodeWsProvider,
    contracts::pool::{self, Pool},
};

pub async fn get_event_logs(
    provider: &NodeWsProvider,
    index_low: u64,
    index_high: u64,
) -> Result<Vec<Log>> {
    let filter = Filter::new()
        .address(pool::eth::get_pool_contract_address())
        .event_signature(vec![
            Pool::Supply::SIGNATURE_HASH,
            Pool::Borrow::SIGNATURE_HASH,
            Pool::Repay::SIGNATURE_HASH,
            Pool::Withdraw::SIGNATURE_HASH,
            Pool::LiquidationCall::SIGNATURE_HASH,
        ])
        .from_block(BlockNumberOrTag::Number(index_low))
        .to_block(BlockNumberOrTag::Number(index_high));
    let logs = provider.get_logs(&filter).await?;

    Ok(logs)
}

/*
Subscribe to realtime event logs for Supply, Borrow, Repay, Withdraw and LiquidationCall events from AAVE
*/
pub async fn subscribe_event_logs(provider: &NodeWsProvider) -> Result<Receiver<Log>> {
    let filter = Filter::new()
        .address(pool::eth::get_pool_contract_address())
        .event_signature(vec![
            Pool::Supply::SIGNATURE_HASH,
            Pool::Borrow::SIGNATURE_HASH,
            Pool::Repay::SIGNATURE_HASH,
            Pool::Withdraw::SIGNATURE_HASH,
            Pool::LiquidationCall::SIGNATURE_HASH,
        ]);
    let mut log_stream = provider.subscribe_logs(&filter).await?;

    let (tx, rx) = tokio::sync::mpsc::channel::<Log>(100);

    tokio::spawn(async move {
        loop {
            match log_stream.recv().await {
                Ok(log) => {
                    println!("New log received: {:#?}", log);
                    tx.send(log);
                }
                Err(err) => {
                    eprintln!("Error receiving log: {:?}", err);
                    break;
                }
            }
        }
    });

    Ok(rx)
}
