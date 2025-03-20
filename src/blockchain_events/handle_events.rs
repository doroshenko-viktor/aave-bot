use std::sync::Arc;

use alloy::rpc::types::Log;
use crossbeam_queue::SegQueue;
use eyre::{OptionExt, Result};
use tokio::sync::mpsc::{Receiver, Sender};
use tracing::{debug, error};

use crate::{
    NodeWsProvider, blockchain_events,
    blockchain_events::events::{
        event_log_parser::parse_event_logs,
        save::{convert_dtos, save_events},
    },
    clients::{
        self,
        events::{get_event_logs, subscribe_event_logs},
    },
    configuration::persistent_config::{ConfigurationKey, config_get, config_set},
    shared::persistence::DbPool,
    utils::iterators::GapSequence64,
};

use super::events::{BlockchainEvent, BlockchainEventPersistenceDto, get::get_events};

pub struct SubscribeEventsLogConfig {
    /**
     * Last block number used for account balance calculation
     */
    pub first_not_handled_block_number: u64,
}

/**
Run stream of not handled AAVE events.
This includes downloaded but not calculated events, not downloaded historical events and real time events.
*/
pub async fn subscribe_blockchain_events(
    provider: &NodeWsProvider,
    db_pool: &DbPool,
    config: SubscribeEventsLogConfig,
) -> Result<Receiver<BlockchainEvent>> {
    let mut event_logs_rx = subscribe_event_logs(provider).await?;

    let notify_real_time_handle_start = Arc::new(tokio::sync::Notify::new());
    let new_logs_buffer: Arc<SegQueue<Log>> = Arc::new(SegQueue::new());
    let (tx, mut loop_back_rx) = tokio::sync::mpsc::channel::<Receiver<Log>>(1);

    // collect new event logs while historical events are handled
    {
        let notify_real_time_handle_start = notify_real_time_handle_start.clone();
        let queue = new_logs_buffer.clone();
        tokio::spawn(async move {
            loop {
                tokio::select! {
                    _ = notify_real_time_handle_start.notified() => {
                        break
                    }
                    log = event_logs_rx.recv() => {
                        match log {
                            Some(log) => {
                                queue.push(log);
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

    let last_blockchain_block_number = clients::blocks::get_current_block(provider).await?;
    handle_new_historical_events(
        db_pool,
        provider,
        &result_tx,
        last_downloaded_block_id + 1,
        last_blockchain_block_number,
    )
    .await?;

    remove_handled_queue_events(new_logs_buffer.clone(), last_blockchain_block_number).await?;
    consume_logs_queue(db_pool, new_logs_buffer.clone(), &result_tx).await;

    notify_real_time_handle_start.notify_one();
    let mut logs_rx = loop_back_rx
        .recv()
        .await
        .expect("Real time Log subscription rx must be defined");

    // consuming queue one more time in case there are some logs added between main part consumed and channel notified to stop fill queue. normally there should be 0 or little event amount there
    consume_logs_queue(db_pool, new_logs_buffer.clone(), &result_tx).await;

    // run new task to listen for log stream and resend parsed events to main events stream and backup them in batches to db
    let db_pool = db_pool.clone();
    tokio::spawn(async move {
        let batch_size = 100;
        let mut events = Vec::with_capacity(batch_size);
        while let Some(log) = logs_rx.recv().await {
            match BlockchainEvent::try_from(&log) {
                Ok(event) => {
                    let persistence_event_dto =
                        BlockchainEventPersistenceDto::try_from(&event).unwrap();
                    result_tx.send(event).await;
                    events.push(persistence_event_dto);

                    if events.len() > batch_size {
                        blockchain_events::save_events(&db_pool, events);
                        config_set(
                            ConfigurationKey::LastDownloadedBlock,
                            &format!("{}", log.block_number.unwrap()),
                        );

                        events = Vec::with_capacity(batch_size);
                    }
                }
                Err(err) => error!(
                    "Couldn't parse BlockchainEvent from log {:#?}\nerror: {}",
                    log, err
                ),
            }
        }
    });

    Ok(result_rx)
}

/*
Removes all events from given queue, which are older than specified block number
*/
async fn remove_handled_queue_events(
    queue: Arc<SegQueue<Log>>,
    last_handled_block_number: u64,
) -> Result<(), eyre::Error> {
    Ok(while let Some(log) = queue.pop() {
        if log
            .block_number
            .ok_or_eyre("Block number must exist for log")?
            > last_handled_block_number
        {
            break;
        }

        queue.pop();
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

/*
Handles TODO:
*/
async fn handle_new_historical_events(
    pool: &DbPool,
    provider: &NodeWsProvider,
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
            let (persistent_events, errors) = convert_dtos(&events);
            // TODO:handle errors
            for event in events {
                result_tx.send(event).await;
            }
            save_events(&pool, persistent_events).await;
        },
    )
}

async fn consume_logs_queue(
    db_pool: &DbPool,
    queue: Arc<SegQueue<Log>>,
    result_tx: &Sender<BlockchainEvent>,
) -> Result<()> {
    loop {
        let logs = {
            if queue.is_empty() {
                drop(queue);
                tokio::task::yield_now().await;
                break;
            }

            let mut logs = Vec::with_capacity(1000);
            while let Some(log) = queue.pop() {
                logs.push(log);
                if logs.len() >= 1000 {
                    break;
                }
            }

            logs
        };
        let (events, errors) = parse_event_logs(&logs);
        // TODO: handle errors
        let (persistent_events, errors) = super::events::save::convert_dtos(&events);
        // TODO:handle errors

        {
            let db_pool = db_pool.clone();
            tokio::spawn(async move {
                save_events(&db_pool, persistent_events).await;
            });
        }

        for event in events {
            result_tx.send(event).await?;
        }
    }

    Ok(())
}
