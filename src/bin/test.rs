use std::{
    num::{NonZero, NonZeroU32},
    str::FromStr,
    sync::Arc,
};

use aave_bot::{
    accounts,
    app::{BlockchainEventService, BlockchainEventsAccountBalancesService},
    blockchain::{
        BlockchainDataProviderFacadeImpl,
        pool_contract::{AavePoolContract, AavePoolContractBlockchainImpl},
    },
    blockchain_events::{self, events::BlockchainEvent},
    configuration::{self, globals, persistent_config::ConfigurationKey},
    operational::aave_events,
    setup,
};
use alloy::primitives::{Address, Uint};
use eyre::Result;
use governor::{Quota, RateLimiter};
use tracing::{error, info, warn};

#[tokio::main]
pub async fn main() -> eyre::Result<()> {
    setup::configure_app().await?;

    let rate_limiter = RateLimiter::direct(
        Quota::per_second(NonZeroU32::new(500).unwrap())
            .allow_burst(NonZeroU32::new(500).unwrap()),
    );

    for i in 0..100 {
        rate_limiter.until_n_ready(NonZero::try_from(500)?).await?;
        println!("Iteration: {}", i);
    }

    // info!("Starting to precalculate user balances...");
    //
    // let config = globals::get_global_settings().clone();
    // let provider = setup::get_new_rpc_provider(&config.rpc_config)?;
    // let pool_contract = AavePoolContractBlockchainImpl::new(
    //     provider.clone(),
    //     Address::from_str(&config.blockchain.aave_v3_pool_contract_address)?,
    // );
    //
    // //
    // let supply: Uint<256, 4> = "0x5f5e100".parse().expect("Invalid hex input");
    //
    // let withdraw: Uint<256, 4> =
    //     "0x5f5e151".parse().expect("Invalid hex input");
    //
    // let res_supply = pool_contract
    //     .get_a_token_valuation(
    //         Address::from_str("0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48")?,
    //         supply,
    //         16498004,
    //     )
    //     .await
    //     .map_err(|e| eyre::eyre!("Failed to get reserve data: {}", e))?;
    //
    // let res_withdraw = pool_contract
    //     .get_a_token_valuation(
    //         Address::from_str("0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48")?,
    //         withdraw,
    //         16498080,
    //     )
    //     .await
    //     .map_err(|e| eyre::eyre!("Failed to get reserve data: {}", e))?;
    // // 99999820 supply
    // // 99999819 withdraw
    // println!(
    //     "A token result on supply: {}",
    //     res_supply
    // );
    // println!(
    //     "A token result on withdraw: {}",
    //     res_withdraw
    // );

    Ok(())
}
