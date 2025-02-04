use std::{cell::RefCell, sync::Arc, time::Duration};

use crate::{
    accounts::data_store::{
        self, get::restore_user_data_store, save::backup_user_data_store, UserDataStore,
    },
    blockchain_events::handle_events::{subscribe_blockchain_events, SubscribeEventsLogConfig},
    clients::{blocks::get_current_block, events::subscribe_event_logs},
    configuration::persistent_config::{config_get, ConfigurationKey},
    shared::persistence::DbPool,
    PubSubNodeProvider,
};
use eyre::Result;
use tokio::sync::RwLock;

// TODO: user data store should be distributed across threads and there should be a separate thread which will save it's content to db every defined period of time. This also saves current handled block number. Plus it should be recoverable from this copy.
// TODO: this should be startup function which runs all necessary functionality including downloading of events, handling user balances and recovering if some calculation was done in previous runs
// TODO: when new token address encountered add it's details to db
pub async fn start(db_pool: &DbPool, provider: &PubSubNodeProvider) -> Result<()> {
    let data_store = get_user_data_store().await?;
    let beginning_block_number = get_current_block(provider).await?;
    let mut events_rx = subscribe_blockchain_events(
        provider,
        db_pool,
        SubscribeEventsLogConfig {
            first_not_handled_block_number: data_store.read().await.get_last_block_number() + 1,
        },
    )
    .await?;

    {
        let data_store = data_store.clone();
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(Duration::from_secs(10)).await;
                backup_user_data_store(data_store.clone()).await;
            }
        });
    }

    while let Some(event) = events_rx.recv().await {
        data_store.borrow_mut().handle_event(&event);
    }

    Ok(())
}

pub async fn get_user_data_store() -> Result<Arc<RwLock<UserDataStore>>> {
    let data_store = restore_user_data_store().await?;
    let data_store = Arc::new(RwLock::new(data_store));

    return Ok(data_store);
}
