use std::{str::FromStr, sync::Arc};

use aave_bot::{
    accounts,
    app::{BlockchainEventService, BlockchainEventsAccountBalancesService},
    blockchain::{
        BlockchainDataProviderFacadeImpl,
        pool_contract::AavePoolContractBlockchainImpl,
    },
    blockchain_events::{self, events::BlockchainEvent},
    configuration::{self, globals, persistent_config::ConfigurationKey},
    operational::aave_events,
    setup,
};
use alloy::primitives::Address;
use eyre::Result;
use reqwest::Url;
use sqlx::Acquire;
use tracing::{error, info, warn};

#[tokio::main]
pub async fn main() -> eyre::Result<()> {
    setup::configure_app().await?;

    info!("Starting to precalculate user balances...");

    let config = globals::get_global_settings().clone();
    let db_pool = globals::get_db_pool();
    let provider = setup::get_new_rpc_provider(&config.rpc_config)?;
    let rpc_client = setup::get_new_rpc_client(Url::from_str(
        &config.rpc_config.node_url,
    )?)?;

    let pool_contract = AavePoolContractBlockchainImpl::new(
        provider.clone(),
        rpc_client.clone(),
        Address::from_str(&config.blockchain.aave_v3_pool_contract_address)?,
    );

    let user_account_store =
        Arc::new(accounts::restore_user_account_store(db_pool).await?);
    let blockchain_facade = BlockchainDataProviderFacadeImpl::new(
        provider,
        rpc_client,
        pool_contract,
    );

    let blockchain_event_parser = BlockchainEventsAccountBalancesService::new(
        user_account_store.clone(),
        blockchain_facade.clone(),
    );

    info!(
        "User account store restored up to block_number: {}",
        user_account_store.get_last_block_number()
    );

    let last_downloaded_block_number =
        get_last_downloaded_block_number().await?;
    info!(
        "Last downloaded block number: {}",
        last_downloaded_block_number
    );

    let (result_tx, mut result_rx) =
        tokio::sync::mpsc::channel::<BlockchainEvent>(100);

    {
        let user_account_store = user_account_store.clone();
        tokio::spawn(async move {
            tokio::task::yield_now().await;
            if let Err(err) = blockchain_events::handle_events::handle_downloaded_historical_events(
                db_pool,
                result_tx,
                blockchain_facade,
                user_account_store.get_last_block_number() + 1,
                last_downloaded_block_number,
            )
            .await {
                error!("Error handling downloaded historical events: {:#?}", err);
            }
        });
    }

    let mut backup_acc = 1u16;
    let x: u16 = 10;
    let mut aave_movements = Vec::with_capacity(x as usize);
    while let Some(event) = result_rx.recv().await {
        if backup_acc % 10 == 0 {
            warn!("Handling event #{}", backup_acc);
        }

        match blockchain_event_parser.handle_event(event.clone()).await {
            Ok(aave_event) => aave_movements.extend(aave_event),
            Err(err) => {
                error!(
                    "Error handling event: {:#?}:\n{}",
                    event, err
                );
                // If there is any error on calculating history this means whole history is invalid
                return Err(err);
            }
        }
        if backup_acc % x == 0 {
            warn!(
                "Backup user data store on event {}...",
                event.block_number
            );
            accounts::backup_user_data_store(&user_account_store).await?;
            aave_events::save_aave_account_movement(db_pool, &aave_movements)
                .await?;
            aave_movements.clear();
        }

        backup_acc += 1;
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
        ConfigurationKey::LastDownloadedTransfersBlock,
    )
    .await?;

    Ok(
        last_downloaded_transfers_block_number
            .min(last_downloaded_block_number),
    )
}
