use crate::shared::persistence::{get_placeholders_str, DbPool};
use eyre::{Error, Result};
use tracing::debug;

use super::{BlockchainEvent, BlockchainEventPersistenceDto};

pub async fn save_events(pool: &DbPool, events: Vec<BlockchainEventPersistenceDto>) -> Result<()> {
    debug!("Inserting {} events to db", events.len());
    let mut query_builder = r#"INSERT INTO blockchain.events (
                block_number,
                log_id,
                event_type_id,
                body
            ) VALUES"#
        .to_string();
    let placeholders = get_placeholders_str(4, events.len());
    query_builder.push_str(&placeholders);
    query_builder.push_str(r#" ON CONFLICT (block_number, log_id) DO NOTHING"#);

    let query = events
        .into_iter()
        .fold(sqlx::query(&query_builder), |query, event| {
            query
                .bind(event.block_number)
                .bind(event.log_number)
                .bind(event.event_type_id)
                .bind(event.body)
        });

    let res = query.execute(pool).await?;
    debug!("Insert liquidation events into db result: {:#?}", res);

    Ok(())
}

pub fn convert_dtos(
    events: &[BlockchainEvent],
) -> (Vec<BlockchainEventPersistenceDto>, Vec<Error>) {
    events.iter().map(|x| x.try_into()).fold(
        (
            Vec::<BlockchainEventPersistenceDto>::new(),
            Vec::<Error>::new(),
        ),
        |mut acc, cur| {
            match cur {
                Ok(x) => acc.0.push(x),
                Err(e) => acc.1.push(e),
            }
            acc
        },
    )
}
