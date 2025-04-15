use eyre::{Error, Result, eyre};
use sqlx::query_as;

use crate::{
    shared::persistence::DbPool, utils::result_utils::CollectionResult,
};

use super::{BlockchainEvent, BlockchainEventPersistenceDto};

pub async fn get_events_paginated(
    db_pool: &DbPool,
    from_block: u64,
    to_block: u64,
    offset: u64,
    limit: u64,
) -> Result<Vec<BlockchainEvent>> {
    let events: Vec<BlockchainEventPersistenceDto> = query_as!(
        BlockchainEventPersistenceDto,
        r#"
        select
            block_number,
            log_number,
            event_type_id,
            body
        from blockchain.events
        where block_number >= $1 and block_number <= $2
            and (body ->> 'user' = '0xd322a49006fc828f9b5b37ab215f99b4e5cab19c'
            or body  ->> 'on_behalf_of' = '0xd322a49006fc828f9b5b37ab215f99b4e5cab19c')
        order by block_number, log_number asc
        offset $3
        limit $4;
        "#,
        from_block as i32,
        to_block as i32,
        offset as i32,
        limit as i32
    )
    .fetch_all(db_pool)
    .await?;

    let events = convert_to_blockchain_events(events);

    if !events.errors.is_empty() {
        return Err(eyre!("")); // TODO: handle error
    }

    Ok(events.results)
}

pub async fn get_events(
    db_pool: &DbPool,
    from_block: u64,
    to_block: u64,
) -> Result<Vec<BlockchainEvent>> {
    // TODO: we get it by block number, but there may be no logs in this diapason,
    // need to get by diapason of returned logs amount
    // until max block number
    let events: Vec<BlockchainEventPersistenceDto> = query_as!(
        BlockchainEventPersistenceDto,
        r#"
        select
            block_number,
            log_number,
            event_type_id,
            body
        from blockchain.events
        where block_number >= $1 and block_number <= $2
        order by block_number, log_number asc;
        "#,
        from_block as i32,
        to_block as i32
    )
    .fetch_all(db_pool)
    .await?;

    let events = convert_to_blockchain_events(events);

    if !events.errors.is_empty() {
        return Err(eyre!("")); // TODO: handle error
    }

    Ok(events.results)
}

fn convert_to_blockchain_events(
    events: Vec<BlockchainEventPersistenceDto>,
) -> CollectionResult<BlockchainEvent> {
    let events = events
        .into_iter()
        .map(|x| {
            let x: BlockchainEvent =
                BlockchainEventPersistenceDto::try_into(x)?;
            Ok(x)
        })
        .fold(
            CollectionResult::<BlockchainEvent>::new(),
            |mut acc, cur| {
                match cur {
                    Ok(event) => acc.results.push(event),
                    Err(error) => acc.errors.push(error),
                }
                acc
            },
        );
    events
}
