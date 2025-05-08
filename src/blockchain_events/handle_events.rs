use std::{fmt::format, sync::Arc};

use alloy::{primitives::Address, rpc::types::Log};
use crossbeam_queue::SegQueue;
use eyre::{OptionExt, Result};
use tokio::sync::{
    Notify,
    mpsc::{Receiver, Sender},
};
use tracing::{debug, error, info, warn};

use crate::{
    NodeWsProvider,
    blockchain_events::{
        self,
        events::{
            event_log_parser::parse_event_logs,
            save::{convert_dtos, save_events},
        },
    },
    clients::{self, events::get_event_logs},
    configuration::persistent_config::{
        ConfigurationKey, config_get, config_set,
    },
    shared::{blockchain::errors::GetEventLogsError, persistence::DbPool},
    tokens::blockchain_client::{
        get_a_token_transfers, get_all_tokens_transfer_logs,
    },
    utils::iterators::GapSequence,
};

use super::events::{
    BlockchainEvent, BlockchainEventPersistenceDto,
    get::{get_events, get_events_paginated},
};

pub struct SubscribeEventsLogConfig {
    /**
     * Last block number used for account balance calculation
     */
    pub first_not_handled_block_number: u64,
}

pub async fn subscribe_full_realtime_event_stream(
    db_pool: &DbPool,
    provider: &NodeWsProvider,
    config: &SubscribeEventsLogConfig,
) -> Result<Receiver<BlockchainEvent>> {
    let (new_logs_buffer, mut rx, notify) =
        spawn_subscribe_realtime_event_logs_with_caching(&provider).await?;

    let last_downloaded_block_id: u64 =
        config_get(ConfigurationKey::LastDownloadedAaveEventsBlock)
            .await?
            .parse()?;

    let (result_tx, result_rx) =
        tokio::sync::mpsc::channel::<BlockchainEvent>(100);

    handle_downloaded_historical_events(
        &db_pool,
        &result_tx,
        config.first_not_handled_block_number + 1,
        last_downloaded_block_id,
    )
    .await
    .unwrap();

    let last_blockchain_block_number =
        match clients::blocks::get_current_block(&provider).await {
            Ok(val) => val,
            Err(err) => {
                error!("Error on get current block: {}", err);
                panic!()
            }
        };

    get_full_historical_events(
        db_pool,
        provider,
        &result_tx,
        last_downloaded_block_id + 1,
        last_blockchain_block_number,
    )
    .await;

    remove_handled_queue_events(
        new_logs_buffer.clone(),
        last_blockchain_block_number,
    )
    .await
    .unwrap();
    consume_logs_queue(
        db_pool,
        new_logs_buffer.clone(),
        &result_tx,
    )
    .await
    .unwrap();

    notify.notify_one();
    let mut logs_rx = rx
        .recv()
        .await
        .expect("Real time Log subscription rx must be defined");

    // consuming queue one more time in case there are some logs added between main part consumed and channel notified to stop fill queue. normally there should be 0 or little event amount there
    consume_logs_queue(
        db_pool,
        new_logs_buffer.clone(),
        &result_tx,
    )
    .await
    .unwrap();

    // listen for log stream and resend parsed events to main events stream and backup them in batches to db
    let batch_size = 100;
    let mut events = Vec::with_capacity(batch_size);
    while let Some(log) = logs_rx.recv().await {
        match BlockchainEvent::try_from(&log) {
            Ok(event) => {
                let persistence_event_dto =
                    BlockchainEventPersistenceDto::try_from(&event).unwrap();
                result_tx.send(event).await.unwrap();
                events.push(persistence_event_dto);

                if events.len() > batch_size {
                    blockchain_events::save_events(db_pool, events)
                        .await
                        .unwrap();
                    config_set(
                        ConfigurationKey::LastDownloadedAaveEventsBlock,
                        &format!("{}", log.block_number.unwrap()),
                    )
                    .await
                    .unwrap();

                    events = Vec::with_capacity(batch_size);
                }
            }
            Err(err) => error!(
                "Couldn't parse BlockchainEvent from log {:#?}\nerror: {}",
                log, err
            ),
        }
    }
    consume_logs_queue(
        db_pool,
        new_logs_buffer.clone(),
        &result_tx,
    )
    .await
    .unwrap();

    notify.notify_one();
    let mut logs_rx = rx
        .recv()
        .await
        .expect("Real time Log subscription rx must be defined");

    // consuming queue one more time in case there are some logs added between main part consumed and channel notified to stop fill queue. normally there should be 0 or little event amount there
    consume_logs_queue(
        db_pool,
        new_logs_buffer.clone(),
        &result_tx,
    )
    .await
    .unwrap();

    // listen for log stream and resend parsed events to main events stream and backup them in batches to db
    {
        let db_pool = db_pool.clone();

        tokio::spawn(async move {
        let batch_size = 100;
        let mut events = Vec::with_capacity(batch_size);
        while let Some(log) = logs_rx.recv().await {
            match BlockchainEvent::try_from(&log) {
                Ok(event) => {
                    let persistence_event_dto =
                        BlockchainEventPersistenceDto::try_from(&event)
                            .unwrap();
                    result_tx.send(event).await.unwrap();
                    events.push(persistence_event_dto);

                    if events.len() > batch_size {
                        blockchain_events::save_events(&db_pool, events)
                            .await
                            .unwrap();
                        config_set(
                            ConfigurationKey::LastDownloadedAaveEventsBlock,
                            &format!("{}", log.block_number.unwrap()),
                        )
                        .await
                        .unwrap();

                        events = Vec::with_capacity(batch_size);
                    }
                }
                Err(err) => error!(
                    "Couldn't parse BlockchainEvent from log {:#?}\nerror: {}",
                    log, err
                ),
            }
        }
        })
        .await
        .unwrap();
    }

    Ok(result_rx)
}

pub async fn spawn_subscribe_realtime_event_logs_with_caching(
    provider: &NodeWsProvider,
) -> Result<(
    Arc<SegQueue<Log>>,
    Receiver<Receiver<Log>>,
    Arc<Notify>,
)> {
    let mut event_logs_rx =
        clients::events::subscribe_event_logs(provider).await?;

    let notify_real_time_handle_start = Arc::new(tokio::sync::Notify::new());
    let new_logs_buffer: Arc<SegQueue<Log>> = Arc::new(SegQueue::new());
    let (tx, mut loop_back_rx) = tokio::sync::mpsc::channel::<Receiver<Log>>(1);

    // collect new event logs while historical events are handled
    {
        let notify_real_time_handle_start =
            notify_real_time_handle_start.clone();
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
            if let Err(e) = tx.send(event_logs_rx).await {
                error!(
                    "Error trying to send event log to main logs channel: {e}"
                )
            }
        });
    }

    Ok((
        new_logs_buffer,
        loop_back_rx,
        notify_real_time_handle_start,
    ))
}

// /**
// Run stream of not handled AAVE events.
// This includes downloaded but not calculated events, not downloaded historical events and real time events.
// */
// pub async fn subscribe_blockchain_events(
//     db_pool: &DbPool,
//     config: SubscribeEventsLogConfig,
// ) -> Result<Receiver<BlockchainEvent>> {
//     // let mut event_logs_rx =
//     //     clients::events::subscribe_event_logs(provider).await?;

//     // let notify_real_time_handle_start = Arc::new(tokio::sync::Notify::new());
//     // let new_logs_buffer: Arc<SegQueue<Log>> = Arc::new(SegQueue::new());
//     // let (tx, mut loop_back_rx) = tokio::sync::mpsc::channel::<Receiver<Log>>(1);

//     // collect new event logs while historical events are handled
//     // {
//     //     let notify_real_time_handle_start =
//     //         notify_real_time_handle_start.clone();
//     //     let queue = new_logs_buffer.clone();
//     //     tokio::spawn(async move {
//     //         loop {
//     //             tokio::select! {
//     //                 _ = notify_real_time_handle_start.notified() => {
//     //                     break
//     //                 }
//     //                 log = event_logs_rx.recv() => {
//     //                     match log {
//     //                         Some(log) => {
//     //                             queue.push(log);
//     //                         }
//     //                         None => break,
//     //                     }
//     //                 }
//     //             }
//     //         }
//     //         if let Err(e) = tx.send(event_logs_rx).await {
//     //             error!(
//     //                 "Error trying to send event log to main logs channel: {e}"
//     //             )
//     //         }
//     //     });
//     // }

//     let last_downloaded_block_id: u64 =
//         config_get(ConfigurationKey::LastDownloadedBlock)
//             .await?
//             .parse()?;

//     let (result_tx, result_rx) =
//         tokio::sync::mpsc::channel::<BlockchainEvent>(100);

//     {
//         let db_pool = db_pool.clone();

//         tokio::spawn(async move {
//             spawn_event_resend_task(
//                 config,
//                 notify_real_time_handle_start,
//                 new_logs_buffer,
//                 loop_back_rx,
//                 last_downloaded_block_id,
//                 result_tx,
//                 db_pool,
//             )
//             .await;
//         });
//     }

//     Ok(result_rx)
// }

// async fn spawn_event_resend_task(
//     config: SubscribeEventsLogConfig,
//     notify_real_time_handle_start: Arc<tokio::sync::Notify>,
//     new_logs_buffer: Arc<SegQueue<Log>>,
//     mut loop_back_rx: Receiver<Receiver<Log>>,
//     last_downloaded_block_id: u64,
//     result_tx: Sender<BlockchainEvent>,
//     db_pool: sqlx::Pool<sqlx::Postgres>,
// ) {
//     let provider =
//         setup::get_new_rpc_provider(&get_global_settings().rpc_config)
//             .await
//             .expect("RPC provider must be created for start event handling");
//     handle_downloaded_historical_events(
//         &db_pool,
//         &result_tx,
//         config.first_not_handled_block_number + 1,
//         last_downloaded_block_id,
//     )
//     .await
//     .unwrap();

//     let last_blockchain_block_number =
//         match clients::blocks::get_current_block(&provider).await {
//             Ok(val) => val,
//             Err(err) => {
//                 error!("Error on get current block: {}", err);
//                 panic!()
//             }
//         };
//     handle_new_historical_events(
//         &db_pool,
//         &provider,
//         &result_tx,
//         last_downloaded_block_id + 1,
//         last_blockchain_block_number,
//     )
//     .await
//     .unwrap();

//     remove_handled_queue_events(
//         new_logs_buffer.clone(),
//         last_blockchain_block_number,
//     )
//     .await
//     .unwrap();
//     consume_logs_queue(
//         &db_pool,
//         new_logs_buffer.clone(),
//         &result_tx,
//     )
//     .await
//     .unwrap();

//     notify_real_time_handle_start.notify_one();
//     let mut logs_rx = loop_back_rx
//         .recv()
//         .await
//         .expect("Real time Log subscription rx must be defined");

//     // consuming queue one more time in case there are some logs added between main part consumed and channel notified to stop fill queue. normally there should be 0 or little event amount there
//     consume_logs_queue(
//         &db_pool,
//         new_logs_buffer.clone(),
//         &result_tx,
//     )
//     .await
//     .unwrap();

//     // listen for log stream and resend parsed events to main events stream and backup them in batches to db
//     let batch_size = 100;
//     let mut events = Vec::with_capacity(batch_size);
//     while let Some(log) = logs_rx.recv().await {
//         match BlockchainEvent::try_from(&log) {
//             Ok(event) => {
//                 let persistence_event_dto =
//                     BlockchainEventPersistenceDto::try_from(&event).unwrap();
//                 result_tx.send(event).await.unwrap();
//                 events.push(persistence_event_dto);

//                 if events.len() > batch_size {
//                     blockchain_events::save_events(&db_pool, events)
//                         .await
//                         .unwrap();
//                     config_set(
//                         ConfigurationKey::LastDownloadedBlock,
//                         &format!("{}", log.block_number.unwrap()),
//                     )
//                     .await
//                     .unwrap();

//                     events = Vec::with_capacity(batch_size);
//                 }
//             }
//             Err(err) => error!(
//                 "Couldn't parse BlockchainEvent from log {:#?}\nerror: {}",
//                 log, err
//             ),
//         }
//     }
// }

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
pub async fn handle_downloaded_historical_events(
    db_pool: &DbPool,
    result_tx: &Sender<BlockchainEvent>,
    first_not_handled_block_number: u64,
    last_downloaded_block_number: u64,
) -> Result<()> {
    let mut current_offset = 0;
    let limit = 1000;
    loop {
        let events = get_events_paginated(
            db_pool,
            first_not_handled_block_number,
            last_downloaded_block_number,
            current_offset,
            limit,
        )
        .await?;
        current_offset += limit;
        debug!("Retrieved {} of events", events.len());

        if events.is_empty() {
            break;
        }

        for event in events {
            result_tx.send(event).await?;
        }
    }

    Ok(())
}

/**
* Retrieves ERC20 asset token transfer logs and AAVE events from the blockchain, stores them in db
* and sends to given channel.
*
* *NOTE:* There are two similar functions but specifically for AAVE events and asset transfers.
* This is done because fetching historical events for transfers and AAVE events is not profficient
* with same block range - this will lead to overuse of Infura API, so preparing historical events
* for account balances calculation is better separately. But when app is in production mode it
* needs all events to be handled in block order. Meaning it needs transfer and AAVE events from one
* block simultaneously. This is done using this particular function.
*
*/
pub async fn get_full_historical_events(
    pool: &DbPool,
    provider: &NodeWsProvider,
    result_tx: &Sender<BlockchainEvent>,
    first_block_number: u64,
    last_block_number: u64,
) -> Result<()> {
    let mut block_sequence = GapSequence::new(
        first_block_number,
        last_block_number,
        10000,
    );
    let mut increase_accumulator: u8 = 0;

    while let Some((left, right)) = block_sequence.next() {
        let transfer_logs =
            match get_all_tokens_transfer_logs(provider, left, right).await {
                Ok(logs) => logs.filter(|log| {
                    let from = Address::from_slice(&log.topics()[1][12..]);
                    let to = Address::from_slice(&log.topics()[2][12..]);

                    from != Address::ZERO && to != Address::ZERO
                }),
                Err(GetEventLogsError::TooManyLogs(Some(
                    next_block_number,
                ))) => {
                    warn!(
                        "Too many logs for block range {} - {}",
                        left, right
                    );
                    block_sequence
                        .fail_interval_with_next(next_block_number - 1)?;
                    continue;
                }
                Err(GetEventLogsError::TooManyLogs(None)) => {
                    warn!(
                        "Too many logs for block range {} - {}",
                        left, right
                    );
                    block_sequence.fail_interval();
                    continue;
                }
                Err(GetEventLogsError::Other(err)) => {
                    error!(
                        "Error on get transfer event logs: {:?}",
                        err
                    );
                    return Err(err);
                }
            };

        let aave_logs = match get_event_logs(provider, left, right).await {
            Ok(logs) => logs,
            Err(GetEventLogsError::TooManyLogs(Some(next_block_number))) => {
                warn!(
                    "Too many logs for block range {} - {}",
                    left, right
                );
                block_sequence
                    .fail_interval_with_next(next_block_number - 1)?;
                continue;
            }
            Err(GetEventLogsError::TooManyLogs(None)) => {
                warn!(
                    "Too many logs for block range {} - {}",
                    left, right
                );
                block_sequence.fail_interval();
                continue;
            }
            Err(GetEventLogsError::Other(err)) => {
                error!("Error on get event logs: {}", err);
                return Err(err);
            }
        };
        if increase_accumulator > 3 {
            block_sequence.success_interval(true);
            increase_accumulator = 0;
        } else {
            block_sequence.success_interval(false);
            increase_accumulator += 1;
        }

        let mut logs = transfer_logs
            .chain(aave_logs.into_iter())
            .collect::<Vec<_>>();
        logs.sort_by_key(|log| log.block_number.unwrap_or_default());

        info!(
            "For block range {} - {} retrieved {} logs",
            left,
            right,
            logs.len()
        );
        if logs.is_empty() {
            continue;
        }
        let parsed_event_logs_result = parse_event_logs(&logs);

        // saving events to db before handling because this is init phase and not performance critical now and to avoid Arc and mutexes later during real time processing
        let persistent_events = convert_dtos(&parsed_event_logs_result.results);
        // TODO:handle errors
        for event in parsed_event_logs_result.results {
            if let Err(e) = result_tx.send(event).await {
                error!(
                    "Error trying to send blockchain events to the channel: {}",
                    e
                );
            }
        }

        if let Err(e) = save_events(pool, persistent_events.results).await {
            error!(
                "Error trying to save events to db: {}",
                e
            );
        }
        config_set(
            ConfigurationKey::LastDownloadedAaveEventsBlock,
            &right.to_string(),
        )
        .await?;
        let x = last_block_number - right;
        warn!("{} blocks left", x)
    }

    Ok(())
}

pub async fn get_historical_asset_transfers(
    pool: &DbPool,
    provider: &NodeWsProvider,
    first_block_number: u64,
    last_block_number: u64,
) -> Result<()> {
    let mut block_sequence = GapSequence::new(
        first_block_number,
        last_block_number,
        250000,
    );
    let mut increase_accumulator: u8 = 0;

    while let Some((left, right)) = block_sequence.next() {
        let mut transfer_logs =
            match get_all_tokens_transfer_logs(provider, left, right).await {
                Ok(logs) => logs
                    .filter(|log| {
                        let from = Address::from_slice(&log.topics()[1][12..]);
                        let to = Address::from_slice(&log.topics()[2][12..]);

                        from != Address::ZERO && to != Address::ZERO
                    })
                    .collect::<Vec<_>>(),
                Err(GetEventLogsError::TooManyLogs(Some(
                    next_block_number,
                ))) => {
                    warn!(
                        "Too many logs for block range {} - {}",
                        left, right
                    );
                    block_sequence
                        .fail_interval_with_next(next_block_number - 1)?;
                    continue;
                }
                Err(GetEventLogsError::TooManyLogs(None)) => {
                    warn!(
                        "Too many logs for block range {} - {}",
                        left, right
                    );
                    block_sequence.fail_interval();
                    continue;
                }
                Err(GetEventLogsError::Other(err)) => {
                    error!(
                        "Error on get transfer event logs: {:?}",
                        err
                    );
                    return Err(err);
                }
            };

        if increase_accumulator > 3 {
            block_sequence.success_interval(true);
            increase_accumulator = 0;
        } else {
            block_sequence.success_interval(false);
            increase_accumulator += 1;
        }

        transfer_logs.sort_by_key(|log| log.block_number.unwrap_or_default());

        info!(
            "For block range {} - {} retrieved {} transfer logs",
            left,
            right,
            transfer_logs.len()
        );

        config_set(
            ConfigurationKey::LastDownloadedTransfersBlock,
            &right.to_string(),
        )
        .await?;

        if transfer_logs.is_empty() {
            continue;
        }
        let parsed_event_logs_result = parse_event_logs(&transfer_logs);

        let persistent_events = convert_dtos(&parsed_event_logs_result.results);

        if let Err(e) = save_events(pool, persistent_events.results).await {
            error!(
                "Error trying to save events to db: {}",
                e
            );
        }
        let x = last_block_number - right;
        warn!("{} blocks left", x)
    }

    Ok(())
}

pub async fn get_historical_aave_events(
    pool: &DbPool,
    provider: &NodeWsProvider,
    first_block_number: u64,
    last_block_number: u64,
) -> Result<()> {
    let mut block_sequence = GapSequence::new(
        first_block_number,
        last_block_number,
        9500,
    );
    while let Some((left, right)) = block_sequence.next() {
        let mut logs = match get_event_logs(provider, left, right).await {
            Ok(logs) => logs,
            Err(GetEventLogsError::TooManyLogs(Some(next_block_number))) => {
                warn!(
                    "Too many logs for block range {} - {}",
                    left, right
                );
                block_sequence
                    .fail_interval_with_next(next_block_number - 1)?;
                continue;
            }
            Err(GetEventLogsError::TooManyLogs(None)) => {
                warn!(
                    "Too many logs for block range {} - {}",
                    left, right
                );
                block_sequence.fail_interval();
                continue;
            }
            Err(GetEventLogsError::Other(err)) => {
                error!("Error on get event logs: {}", err);
                return Err(err);
            }
        };

        block_sequence.success_interval(true);

        logs.sort_by_key(|log| log.block_number.unwrap_or_default());

        info!(
            "For block range {} - {} retrieved {} transfer logs",
            left,
            right,
            logs.len()
        );

        config_set(
            ConfigurationKey::LastDownloadedAaveEventsBlock,
            &right.to_string(),
        )
        .await?;

        if logs.is_empty() {
            continue;
        }
        let parsed_event_logs_result = parse_event_logs(&logs);
        if !parsed_event_logs_result.errors.is_empty() {
            let e = parsed_event_logs_result
                .errors
                .iter()
                .map(|e| format!("{:?}", e))
                .collect::<Vec<_>>()
                .join("\n");
            error!(
                "Error parsing event logs to BlockchainEvents: {}",
                e
            )
        }

        let persistent_events = convert_dtos(&parsed_event_logs_result.results);

        if !persistent_events.errors.is_empty() {
            let e = persistent_events
                .errors
                .iter()
                .map(|e| format!("{:?}", e))
                .collect::<Vec<_>>()
                .join("\n");
            error!(
                "Error parding event logs to BlockchainEvents: {}",
                e
            )
        }
        if let Err(e) = save_events(pool, persistent_events.results).await {
            error!(
                "Error trying to save events to db: {}",
                e
            );
        }
        let x = last_block_number - right;
        warn!("{} blocks left", x)
    }

    Ok(())
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
        let parsed_events = parse_event_logs(&logs);
        // TODO: handle errors
        let persistent_events =
            super::events::save::convert_dtos(&parsed_events.results);
        // TODO:handle errors

        {
            let db_pool = db_pool.clone();
            tokio::spawn(async move {
                save_events(&db_pool, persistent_events.results).await;
            });
        }

        for event in parsed_events.results {
            result_tx.send(event).await?;
        }
    }

    Ok(())
}
