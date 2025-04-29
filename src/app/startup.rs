use std::{sync::Arc, time::Duration};

use super::{config::AppConfig, tasks};
use crate::{
    NodeWsProvider,
    accounts::{self, UserAccountStore},
    blockchain::BlockchainDataProviderFacade,
    blockchain_events, price_feed,
    shared::persistence::DbPool,
};
use eyre::Result;

/*
Start full app functionality with spawning all background jobs and handlers
*/
pub async fn start_full_app<T>(
    db_pool: DbPool,
    provider: NodeWsProvider,
    blockchain_facade: T,
    config: AppConfig,
) -> Result<()>
where
    T: BlockchainDataProviderFacade + Send + Sync + 'static,
{
    let user_account_store = accounts::restore_user_account_store(&db_pool)
        .await?
        .build(blockchain_facade);

    let user_account_store = Arc::new(user_account_store);

    start_account_balances_tracking(
        db_pool,
        provider,
        user_account_store.clone(),
    )
    .await?;

    tasks::spawn_user_accounts_store_backup_task(
        user_account_store.clone(),
        Duration::from_secs(30), // TODO: move to config
    );

    // TODO: task to call aave for exact account balance to ensure precision(mostly because of APR changes)

    tasks::spawn_low_hf_accounts_refresh_task(
        user_account_store.clone(),
        Duration::from_secs(5), // TODO: move literal to config
    );
    start_price_feeds(user_account_store, config).await?;

    Ok(())
}

pub async fn start_price_feeds<T>(
    user_account_store: Arc<UserAccountStore<T>>,
    config: AppConfig,
) -> Result<()>
where
    T: BlockchainDataProviderFacade + Send + Sync + 'static,
{
    let (ws_connection_manager, prices_rx) =
        price_feed::start_price_feed_consumption(&config).await?;
    price_feed::start_configured_price_feeds(ws_connection_manager).await?;
    tasks::spawn_handling_price_changes(user_account_store.clone(), prices_rx)
        .await?;

    Ok(())
}

/**
Subscribes to price feeds from supported sources and runs the corresponding handlers for these feeds.
*/
pub async fn start_account_balances_tracking<T>(
    db_pool: DbPool,
    provider: NodeWsProvider,
    user_account_store: Arc<UserAccountStore<T>>,
) -> Result<(), eyre::Error>
where
    T: BlockchainDataProviderFacade + Send + Sync + 'static,
{
    let aave_events_rx =
        blockchain_events::handle_events::subscribe_full_realtime_event_stream(
            &db_pool,
            &provider,
            &blockchain_events::SubscribeEventsLogConfig {
                first_not_handled_block_number: user_account_store
                    .get_last_block_number()
                    + 1,
            },
        )
        .await?;
    tasks::start_handling_account_state_changes(
        user_account_store.clone(),
        aave_events_rx,
    )
    .await?;

    Ok(())
}
