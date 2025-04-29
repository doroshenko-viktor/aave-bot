use std::{str::FromStr, sync::Arc};

use aave_bot::{
    accounts,
    blockchain::{
        BlockchainDataProviderFacadeImpl,
        pool_contract::AavePoolContractBlockchainImpl,
    },
    blockchain_events::{self, events::BlockchainEvent},
    configuration::{self, globals, persistent_config::ConfigurationKey},
    setup,
};
use alloy::primitives::Address;
use eyre::Result;
use tracing::{error, info, warn};

#[tokio::main]
pub async fn main() -> eyre::Result<()> {
    setup::configure_app().await?;

    info!("Starting to precalculate user balances...");
    let config = globals::get_global_settings().clone();
    let db_pool = globals::get_db_pool();
    let provider = setup::get_new_rpc_provider(&config.rpc_config)?;
    let pool_contract = AavePoolContractBlockchainImpl::new(
        provider.clone(),
        Address::from_str(&config.blockchain.aave_v3_pool_contract_address)?,
    );

    let blockchain_facade =
        BlockchainDataProviderFacadeImpl::new(provider, pool_contract);

    let user_account_store = Arc::new(
        accounts::restore_user_account_store(db_pool)
            .await?
            .build(blockchain_facade),
    );
    let last_downloaded_block_number =
        get_last_downloaded_block_number().await?;

    let (result_tx, mut result_rx) =
        tokio::sync::mpsc::channel::<BlockchainEvent>(100);

    {
        let user_account_store = user_account_store.clone();
        tokio::spawn(async move {
            tokio::task::yield_now().await;
            if let Err(err) = blockchain_events::handle_events::handle_downloaded_historical_events(
                db_pool,
                &result_tx,
                user_account_store.get_last_block_number() + 1,
                last_downloaded_block_number,
            )
            .await {
                error!("Error handling downloaded historical events: {:#?}", err);
            }
        });
    }

    while let Some(event) = result_rx.recv().await {
        if let Err(err) = user_account_store.handle_event(event.clone()).await {
            error!(
                "Error handling event: {:#?}:\n{}",
                event, err
            );
            // If there is any error on calculating history this means whole history is invalid
            return Err(err);
        }
    }

    accounts::backup_user_data_store(&user_account_store).await?;

    Ok(())
}

async fn get_last_downloaded_block_number() -> Result<u64> {
    let last_downloaded_block_number = configuration::get_as::<u64>(
        ConfigurationKey::LastDownloadedAaveEventsBlock,
    )
    .await?;

    let last_downloaded_transfers_block_number = configuration::get_as::<u64>(
        ConfigurationKey::LastDownloadedAaveEventsBlock,
    )
    .await?;

    Ok(
        last_downloaded_transfers_block_number
            .min(last_downloaded_block_number),
    )
}
