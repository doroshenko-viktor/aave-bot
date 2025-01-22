use super::WithdrawEventDto;
use crate::shared::persistence::{get_placeholders_str, PostgresPool, SCHEMA_BLOCKCHAIN};
use eyre::Result;
use tracing::debug;

pub async fn save_withdraw_events(
    pool: &PostgresPool,
    events: Vec<WithdrawEventDto>,
) -> Result<()> {
    debug!("Inserting {} withdraw events into db", events.len());
    let mut query_builder = format!(
        r#"INSERT INTO {}.withdraw_events (
                block_number,
                log_id,
                address,
                reserve,
                "user",
                "to",
                amount
            ) VALUES"#,
        SCHEMA_BLOCKCHAIN
    );
    let placeholders = get_placeholders_str(7, events.len());
    query_builder.push_str(&placeholders);
    query_builder.push_str(r#" ON CONFLICT (block_number, log_id) DO NOTHING"#);

    let query = events
        .into_iter()
        .fold(sqlx::query(&query_builder), |query, event| {
            query
                .bind(event.block_number as i32)
                .bind(event.log_id)
                .bind(event.address)
                .bind(event.reserve)
                .bind(event.user)
                .bind(event.to)
                .bind(event.amount)
        });

    let res = query.execute(pool).await?;
    debug!("Insert withdraw event result: {:#?}", res);

    Ok(())
}
