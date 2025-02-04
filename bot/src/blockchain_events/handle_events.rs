use std::{collections::VecDeque, sync::Arc};

use alloy::{rpc::types::Log, sol_types::SolEvent};
use eyre::{OptionExt, Result};
use futures::SinkExt;
use sqlx::{postgres::PgPoolCopyExt, PgPool};
use tokio::sync::{
    mpsc::{Receiver, Sender},
    Mutex, Notify,
};
use tokio_tungstenite::tungstenite::http::header::X_CONTENT_TYPE_OPTIONS;
use tracing::debug;

use crate::{
    accounts::data_store::UserDataStore,
    blockchain_events::events::{event_log_parser::parse_event_logs, save::save_events},
    clients::{
        self,
        events::{get_event_logs, subscribe_event_logs},
    },
    configuration::persistent_config::{config_get, config_set, ConfigurationKey},
    shared::persistence::DbPool,
    utils::iterators::GapSequence64,
    PubSubNodeProvider,
};

use super::events::{
    event_log_parser::parse_event_log, get::get_events, BlockchainEvent,
    BlockchainEventPersistenceDto,
};

pub struct SubscribeEventsLogConfig {
    /**
     * Last block number used for account balance calculation
     */
    pub first_not_handled_block_number: u64,
}
pub async fn subscribe_blockchain_events(
    provider: &PubSubNodeProvider,
    db_pool: &DbPool,
    config: SubscribeEventsLogConfig,
) -> Result<Receiver<BlockchainEvent>> {
    let mut event_logs_rx = subscribe_event_logs(provider).await?;

    let notify_real_time_handle_start = Arc::new(tokio::sync::Notify::new());
    let queue: Arc<Mutex<VecDeque<Log>>> = Arc::new(Mutex::new(VecDeque::new()));
    let (tx, mut loop_back_rx) = tokio::sync::mpsc::channel::<Receiver<Log>>(1);

    {
        let notify_real_time_handle_start = notify_real_time_handle_start.clone();
        let mut queue = queue.clone();
        tokio::spawn(async move {
            loop {
                tokio::select! {
                    _ = notify_real_time_handle_start.notified() => {
                        break
                    }
                    log = event_logs_rx.recv() => {
                        match log {
                            Some(log) => {
                                let mut queue = queue.lock().await;
                                queue.push_back(log);
                            }
                            None => break,
                        }
                    }
                }
            }
            tx.send(event_logs_rx);
        });
    }

    let last_downloaded_block_id: u64 = config_get(ConfigurationKey::LastDownloadedBlock)
        .await?
        .parse()?;

    let (result_tx, result_rx) = tokio::sync::mpsc::channel::<BlockchainEvent>(100);

    handle_downloaded_historical_events(
        db_pool,
        &result_tx,
        config.first_not_handled_block_number,
        last_downloaded_block_id,
    )
    .await?;

    let last_existing_block_number = clients::blocks::get_current_block(provider).await?;
    handle_new_historical_events(
        db_pool,
        provider,
        &result_tx,
        last_downloaded_block_id + 1,
        last_existing_block_number,
    )
    .await?;

    remove_handled_queue_events(queue.clone(), last_existing_block_number).await?;
    consume_logs_queue(db_pool, queue.clone(), &result_tx).await;

    notify_real_time_handle_start.notify_one();
    let mut rx = loop_back_rx
        .recv()
        .await
        .expect("Real time Log subscription rx must be defined");

    // consuming queue one more time in case there are some logs added between main part consumed and channel notified to stop fill queue. normally there should be 0 or little event amount there
    consume_logs_queue(db_pool, queue.clone(), &result_tx).await;

    tokio::spawn(async {
        let mut events = Vec::with_capacity(100);
        while let Some(log) = rx.recv().await {
            let event = parse_event_log(&log)?;
            let persistence_event_dto = BlockchainEventPersistenceDto::try_from(&event)?;
            result_tx.send(event).await;
            events.push(persistence_event_dto);

            if events.len() > 100 {
                save_events(db_pool, events);
                config_set(
                    ConfigurationKey::LastDownloadedBlock,
                    &format!("{}", log.block_number.unwrap()),
                );

                events = Vec::new();
            }
        }
    });

    Ok(result_rx)
}

async fn remove_handled_queue_events(
    queue: Arc<Mutex<VecDeque<Log>>>,
    last_handled_block_number: u64,
) -> Result<(), eyre::Error> {
    let mut queue = queue.lock().await;
    Ok(while let Some(log) = queue.front() {
        if log
            .block_number
            .ok_or_eyre("Block number must exist for log")?
            > last_handled_block_number
        {
            break;
        }

        queue.pop_front();
    })
}

/**
* Sends events to the handling queue starting from last handled event to the last downloaded.
*/
async fn handle_downloaded_historical_events(
    db_pool: &DbPool,
    result_tx: &Sender<BlockchainEvent>,
    first_not_handled_block_number: u64,
    last_downloaded_block_number: u64,
) -> Result<()> {
    for (left, right) in GapSequence64::new(
        first_not_handled_block_number,
        last_downloaded_block_number,
        9000,
    ) {
        let events = get_events(db_pool, left, right).await?;

        for event in events {
            result_tx.send(event).await;
        }
    }
    Ok(())
}

async fn handle_new_historical_events(
    pool: &DbPool,
    provider: &alloy::providers::RootProvider<alloy::pubsub::PubSubFrontend>,
    result_tx: &Sender<BlockchainEvent>,
    last_downloaded_block_id: u64,
    beginning_block_number: u64,
) -> Result<(), eyre::Error> {
    Ok(
        for (left, right) in
            GapSequence64::new(last_downloaded_block_id + 1, beginning_block_number, 9000)
        {
            let logs = get_event_logs(provider, left, right).await?;
            debug!(
                "For block range {} - {} retrieved {} logs",
                left,
                right,
                logs.len()
            );
            if logs.is_empty() {
                continue;
            }
            let (events, errors) = parse_event_logs(&logs);

            let pool = pool.clone();
            // saving events to db before handling because this is init phase and not performance critical now and to avoid Arc and mutexes later during real time processing
            save_events(&pool, &events).await;

            for event in events {
                result_tx.send(event).await;
            }
        },
    )
}

async fn consume_logs_queue(
    db_pool: &DbPool,
    queue: Arc<Mutex<VecDeque<Log>>>,
    result_tx: &Sender<BlockchainEvent>,
) -> Result<()> {
    loop {
        let mut queue = queue.lock().await;
        let logs = {
            if queue.is_empty() {
                drop(queue);
                tokio::task::yield_now().await;
                break;
            }

            let mut logs = Vec::with_capacity(1000);
            while let Some(log) = queue.pop_front() {
                logs.push(log);
                if logs.len() >= 1000 {
                    break;
                }
            }

            logs
        };
        let (events, errors) = parse_event_logs(&logs);
        // TODO: handle errors
        save_events(&db_pool, &events).await;
        for event in events {
            result_tx.send(event).await?;
        }
    }

    Ok(())
}
