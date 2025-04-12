
use aave_bot::{
    configuration::globals::get_rpc_provider,
    contracts::chainlink::{Chainlink, ChainlinkInterface},
    setup::{Application, configure_app},
};
use alloy::{
    primitives::Address,
    providers::Provider,
};
use eyre::Result;

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
