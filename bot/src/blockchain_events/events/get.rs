use eyre::{eyre, Error, Result};
use sqlx::query_as;

use crate::shared::persistence::DbPool;

use super::{BlockchainEvent, BlockchainEventPersistenceDto};

pub async fn get_events(
    db_pool: &DbPool,
    from_block: u64,
    to_block: u64,
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
        order by block_number, log_number asc;
        "#,
        from_block as i32,
        to_block as i32
    )
    .fetch_all(db_pool)
    .await?;

    let (events, errors) = events
        .into_iter()
        .map(|x| {
            let x: BlockchainEvent = BlockchainEventPersistenceDto::try_into(x)?;
            Ok(x)
        })
        .fold(
            (Vec::<BlockchainEvent>::new(), Vec::<Error>::new()),
            |mut acc, cur| {
                match cur {
                    Ok(event) => acc.0.push(event),
                    Err(error) => acc.1.push(error),
                }
                acc
            },
        );

    if !errors.is_empty() {
        return Err(eyre!("")); // TODO: handle error
    }

    Ok(events)
}
