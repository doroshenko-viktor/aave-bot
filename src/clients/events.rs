use std::num::NonZero;

use alloy::{
    eips::BlockNumberOrTag,
    providers::Provider,
    rpc::types::{Filter, Log},
    sol_types::SolEvent,
};
use eyre::Result;
use tokio::sync::mpsc::Receiver;
use tracing::{debug, error, info};

use crate::{
    NodeWsProvider,
    configuration::globals,
    contracts::pool::{self, Pool},
    shared::blockchain::errors::{GetEventLogsError, TooManyLogsErrPayload},
};

pub async fn get_event_logs(
    provider: &NodeWsProvider,
    index_low: u64,
    index_high: u64,
) -> Result<Vec<Log>, GetEventLogsError> {
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
    globals::RPC_RATE_LIMITER
        .until_n_ready(NonZero::new(255).unwrap())
        .await
        .map_err(eyre::Report::from)?;
    debug!(
        "Requesting portion of event logs between blocks: {} - {}",
        index_low, index_high
    );
    match provider.get_logs(&filter).await {
        Ok(logs) => Ok(logs),
        Err(alloy::transports::RpcError::ErrorResp(error_payload)) => {
            if let Some(data) = error_payload.data {
                let data = data.to_string();
                let data = serde_json::from_str::<TooManyLogsErrPayload>(&data)
                    .unwrap();
                error!("DATA: {:#?}", data);
                let to_block = u64::from_str_radix(&data.to[2..], 16).unwrap();

                Err(GetEventLogsError::TooManyLogs(Some(
                    to_block,
                )))
            } else {
                Err(GetEventLogsError::TooManyLogs(None))
            }
        }
        Err(other) => Err(GetEventLogsError::Other(
            eyre::Report::from(other),
        )),
    }
}

/*
Subscribe to realtime event logs for Supply, Borrow, Repay, Withdraw and LiquidationCall events from AAVE
*/
pub async fn subscribe_event_logs(
    provider: &NodeWsProvider,
) -> Result<Receiver<Log>> {
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

    // TODO: probably I don't need additional task for this
    tokio::spawn(async move {
        loop {
            match log_stream.recv().await {
                Ok(log) => {
                    println!("New log received: {:#?}", log);
                    if let Err(e) = tx.send(log).await {
                        error!(
                            "Error sending received log to log channel: {}",
                            e
                        );
                    }
                }
                Err(err) => {
                    error!("Error receiving log: {:?}", err);
                    break;
                }
            }
        }
    });

    Ok(rx)
}
