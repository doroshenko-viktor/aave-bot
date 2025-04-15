use crate::NodeHttpProvider;
use crate::configuration::globals;
use crate::shared::blockchain::errors::{
    GetEventLogsError, TooManyLogsErrPayload,
};
use alloy::providers::Provider;
use alloy::rpc::types::{Filter, Log};
use eyre::Result;
use std::num::NonZero;
use tracing::error;

pub async fn get_logs(
    provider: &NodeHttpProvider,
    filter: &Filter,
) -> Result<Vec<Log>, GetEventLogsError> {
    globals::RPC_RATE_LIMITER
        .until_n_ready(NonZero::new(255).unwrap())
        .await
        .map_err(eyre::Report::from)?;

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
