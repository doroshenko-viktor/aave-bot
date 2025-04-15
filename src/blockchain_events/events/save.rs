use crate::{
    shared::persistence::{DbPool, get_placeholders_str},
    utils::result_utils::CollectionResult,
};
use eyre::{Error, Result};
use tracing::debug;

use super::{BlockchainEvent, BlockchainEventPersistenceDto};

pub async fn save_events(
    pool: &DbPool,
    events: Vec<BlockchainEventPersistenceDto>,
) -> Result<()> {
    debug!(
        "Inserting {} events to db",
        events.len()
    );
    let mut query_builder = r#"INSERT INTO blockchain.events (
                block_number,
                log_number,
                event_type_id,
                body
            ) VALUES"#
        .to_string();
    let placeholders = get_placeholders_str(4, events.len());
    query_builder.push_str(&placeholders);
    query_builder
        .push_str(r#" ON CONFLICT (block_number, log_number) DO NOTHING"#);

    let query = events.into_iter().fold(
        sqlx::query(&query_builder),
        |query, event| {
            query
                .bind(event.block_number)
                .bind(event.log_number)
                .bind(event.event_type_id)
                .bind(event.body)
        },
    );

    let res = query.execute(pool).await?;
    debug!(
        "Inserted events into db result: {:#?}",
        res
    );

    Ok(())
}

pub fn convert_dtos(
    events: &[BlockchainEvent],
) -> CollectionResult<BlockchainEventPersistenceDto> {
    events.iter().map(|x| x.try_into()).fold(
        CollectionResult::new(),
        |mut acc, cur| {
            match cur {
                Ok(x) => acc.results.push(x),
                Err(e) => acc.errors.push(e),
            }
            acc
        },
    )
}
