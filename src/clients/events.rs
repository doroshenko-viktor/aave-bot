use std::num::NonZero;

use alloy::{
    eips::BlockNumberOrTag,
    providers::Provider,
    rpc::types::{Filter, Log},
    sol_types::SolEvent,
};
use eyre::{Result, eyre};
use serde::Deserialize;
use tokio::sync::mpsc::Receiver;
use tracing::{debug, error, info};

use crate::{
    NodeWsProvider,
    configuration::globals,
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
    globals::RPC_RATE_LIMITER
        .until_n_ready(NonZero::new(255).unwrap())
        .await?;
    debug!(
        "Requesting portion of event logs between blocks: {} - {}",
        index_low, index_high
    );
    match provider.get_logs(&filter).await {
        Ok(logs) => Ok(logs),
        Err(e) => {
            error!("Error on logs call: {:#?}", e);
            match e {
                // 2025-04-06T21:29:37.321920Z ERROR aave_bot::clients::events: Error on logs call: ErrorResp(
                //     ErrorPayload {
                //         code: -32005,
                //         message: "query returned more than 10000 results. Try with this block range [0x13823C1, 0x13835FF].",
                //         data: Some(
                //             RawValue(
                //                 {"from":"0x13823C1","limit":10000,"to":"0x13835FF"},
                //             ),
                //         ),
                //     },
                // )
                alloy::transports::RpcError::ErrorResp(error_payload) => {
                    if let Some(data) = error_payload.data {
                        #[derive(Debug, Deserialize)]
                        struct VALUE {
                            from: String,
                            to: String,
                        }
                        let data = data.to_string();
                        let data =
                            serde_json::from_str::<VALUE>(&data).unwrap();
                        error!("DATA: {:#?}", data);
                        let from_block =
                            u64::from_str_radix(&data.from[2..], 16).unwrap();
                        let to_block =
                            u64::from_str_radix(&data.to[2..], 16).unwrap();

                        error!("from: {}; to: {}", from_block, to_block);
                    }
                    panic!()
                }
                alloy::transports::RpcError::NullResp => todo!(),
                alloy::transports::RpcError::UnsupportedFeature(_) => todo!(),
                alloy::transports::RpcError::LocalUsageError(error) => todo!(),
                alloy::transports::RpcError::SerError(error) => todo!(),
                alloy::transports::RpcError::DeserError { err, text } => {
                    todo!()
                }
                alloy::transports::RpcError::Transport(_) => todo!(),
            }
        }
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
