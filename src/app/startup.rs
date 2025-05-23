use super::{
    BlockchainEventService, config::AppConfig,
    price_changes_handler::PriceChangesHandler, tasks,
};
use crate::{
    NodeWsProvider,
    accounts::UserAccountStore,
    blockchain::BlockchainDataProviderFacade,
    blockchain_events::{self, SubscribeEventsLogConfig},
    price_feed::{self, PriceFeedConfig},
    shared::persistence::DbPool,
};
use eyre::Result;
use std::{sync::Arc, time::Duration};

/*
Start full app functionality with spawning all background jobs and handlers
*/
pub async fn start_full_app<T>(
    db_pool: &'static DbPool,
    provider: NodeWsProvider,
    user_account_store: Arc<UserAccountStore>,
    blockchain_event_parser: impl BlockchainEventService + 'static,
    price_changes_handler: impl PriceChangesHandler + Send + Sync + 'static,
    blockchain_facade: impl BlockchainDataProviderFacade + 'static,
    config: AppConfig,
) -> Result<()> {
    start_account_balances_tracking(
        db_pool,
        provider,
        blockchain_event_parser,
        blockchain_facade,
        SubscribeEventsLogConfig {
            first_not_handled_block_number: user_account_store
                .get_last_block_number()
                + 1,
        },
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
    start_price_feeds(
        price_changes_handler,
        PriceFeedConfig::from(&config),
    )
    .await?;

    Ok(())
}

pub async fn start_price_feeds<T>(
    price_changes_handler: T,
    config: PriceFeedConfig,
) -> Result<()>
where
    T: PriceChangesHandler + Send + Sync + 'static,
{
    let (ws_connection_manager, prices_rx) =
        price_feed::start_price_feed_consumption(&config).await?;
    price_feed::start_configured_price_feeds(ws_connection_manager).await?;
    tasks::spawn_handling_price_changes(price_changes_handler, prices_rx)
        .await?;

    Ok(())
}

/**
Subscribes to price feeds from supported sources and runs the corresponding handlers for these feeds.
*/
pub async fn start_account_balances_tracking(
    db_pool: &DbPool,
    provider: NodeWsProvider,
    blockchain_event_service: impl BlockchainEventService + 'static,
    blockchain_facade: impl BlockchainDataProviderFacade + 'static,
    config: SubscribeEventsLogConfig,
) -> Result<(), eyre::Error> {
    let aave_events_rx =
        blockchain_events::handle_events::subscribe_full_realtime_event_stream(
            db_pool,
            &provider,
            &config,
            blockchain_facade,
        )
        .await?;
    tasks::start_handling_account_state_changes(
        blockchain_event_service,
        aave_events_rx,
    )
    .await?;

    Ok(())
}
