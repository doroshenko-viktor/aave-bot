use std::{ops::Deref, time::Duration};

use super::app_orchestrator;
use crate::{
    NodeWsProvider,
    accounts::{self, user_account_store::data_store::UserAccountStore},
    blockchain_events::handle_events::{SubscribeEventsLogConfig, subscribe_blockchain_events},
    configuration::{self},
    price_feed::{self, PriceFeedConfig, PriceFeedsService, ticker_service},
    shared::persistence::DbPool,
    tokens::asset_rules_aggregator::AssetRules,
};
use eyre::Result;

pub struct Config {
    pub price_feed_config: PriceFeedConfig,
}

impl Deref for Config {
    type Target = PriceFeedConfig;

    fn deref(&self) -> &Self::Target {
        &self.price_feed_config
    }
}

// TODO: user data store should be distributed across threads and there should be a separate thread which will save it's content to db every defined period of time. This also saves current handled block number. Plus it should be recoverable from this copy.
// TODO: this should be startup function which runs all necessary functionality including downloading of events, handling user balances and recovering if some calculation was done in previous runs
// TODO: when new token address encountered add it's details to db and token aggregator
// TODO: for all tokens, which are interesting for liquidation need to run price feed through ws_connection_manager
pub async fn start_application(
    db_pool: DbPool,
    provider: NodeWsProvider,
    config: Config,
) -> Result<()> {
    let (mut ws_connection_manager, prices_rx) =
        price_feed::start_price_feed_consumption(&config).await?;

    start_price_feeds(ws_connection_manager).await?;

    let user_account_store = accounts::restore_user_account_store(&db_pool).await?;

    start_periodical_full_account_refresh(user_account_store.clone());

    let mut blockchain_events_rx = subscribe_blockchain_events(
        &provider,
        &db_pool,
        SubscribeEventsLogConfig {
            first_not_handled_block_number: user_account_store.get_last_block_number() + 1,
        },
    )
    .await?;

    app_orchestrator::start_handling_account_state_changes(
        user_account_store.clone(),
        blockchain_events_rx,
    )
    .await;

    app_orchestrator::start_handling_price_changes(user_account_store.clone(), prices_rx).await;

    start_user_data_store_periodical_backup(user_account_store);

    Ok(())
}

// TODO: create configuration in db for active tickers with representations for specific feeds, active tokens for collateral and debt and mechanism to keep track only events and accounts for these tokens
async fn start_price_feeds(mut price_feeds_service: PriceFeedsService) -> Result<()> {
    Ok(for ticker in ticker_service::get_active_tickers() {
        price_feeds_service.start_feed(ticker.1).await;
    })
}

fn start_periodical_full_account_refresh(data_store: UserAccountStore) {
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(Duration::from_secs(5)).await;
            data_store.refresh_low_hf_accounts();
        }
    });
}

fn start_user_data_store_periodical_backup(user_data_store: UserAccountStore) {
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(Duration::from_secs(30)).await;
            accounts:: backup_user_data_store(&user_data_store).await;
        }
    });
}
