use alloy::{
    eips::BlockNumberOrTag,
    providers::{Provider, RootProvider},
    pubsub::PubSubFrontend,
    rpc::types::{Filter, Log},
    sol_types::SolEvent,
};
use eyre::Error;

use crate::contracts::pool::{self, Pool};

pub async fn get_event_logs(
    provider: &RootProvider<PubSubFrontend>,
    index_low: u64,
    index_high: u64,
) -> Result<Vec<Log>, Error> {
    let filter = Filter::new()
        .address(pool::eth::get_pool_contract_address())
        .event_signature(Pool::Borrow::SIGNATURE_HASH)
        // .event_signature(Pool::Repay::SIGNATURE_HASH)
        // .event_signature(Pool::Withdraw::SIGNATURE_HASH)
        // .event_signature(Pool::LiquidationCall::SIGNATURE_HASH)
        .from_block(BlockNumberOrTag::Number(index_low))
        .to_block(BlockNumberOrTag::Number(index_high));
    let logs = provider.get_logs(&filter).await?;

    Ok(logs)
}
