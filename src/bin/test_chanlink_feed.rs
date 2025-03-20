use std::str::FromStr;

use aave_bot::{
    NodeWsProvider,
    configuration::globals::get_rpc_provider,
    contracts::chainlink::{Chainlink, ChainlinkInterface},
    setup::{Application, configure_app},
    utils::datetime::convert_timestamp_to_string,
};
use alloy::{
    consensus::Transaction,
    contract::ContractInstance,
    dyn_abi::JsonAbiExt,
    json_abi::{Function, JsonAbi},
    primitives::Address,
    providers::Provider,
    sol_types::SolValue,
};
use alloy_contract::Interface;
use eyre::Result;
use futures::StreamExt;
use serde_json::from_str;
use tracing::{error, info, warn};

const BTCUSD_PRICE_FEED_ADDRESS: &str = "0x5f4eC3Df9cbd43714FE2740f5E3616155c5b8419";

#[tokio::main]
pub async fn main() -> Result<()> {
    let Application { rpc_rate_limiter } = configure_app().await?;
    let rpc_provider = get_rpc_provider();
    let address: Address = BTCUSD_PRICE_FEED_ADDRESS.parse()?;
    let chainlink_interface: ChainlinkInterface = Chainlink::new(address, rpc_provider.clone());
    rpc_provider.subscribe_full_pending_transactions();

    // let chainlink = Chainlink::new(address, rpc_provider);
    // let latest_round_data = chainlink.latestRoundData().call().await?;

    // info!("answer: {}", latest_round_data.answer);
    // info!("round_id: {}", latest_round_data.roundId);
    // info!(
    //     "date: {}",
    //     convert_timestamp_to_string(latest_round_data.updatedAt.try_into()?)
    // );

    Ok(())
}
