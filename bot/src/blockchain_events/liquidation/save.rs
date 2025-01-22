use crate::shared::persistence::{get_placeholders_str, PostgresPool, SCHEMA_BLOCKCHAIN};
use eyre::Result;
use tracing::debug;

use super::LiquidationEventDto;

pub async fn save_liquidation_events(
    pool: &PostgresPool,
    events: Vec<LiquidationEventDto>,
) -> Result<()> {
    debug!("Inserting {} liquidation events to db", events.len());
    let mut query_builder = format!(
        r#"INSERT INTO {}.liquidation_events (
                block_number,
                log_id,
                collateral_asset,
                debt_asset,
                "user",
                debt_to_cover,
                liquidated_collateral_amount,
                liquidator,
                receive_a_token
            ) VALUES"#,
        SCHEMA_BLOCKCHAIN
    );
    let placeholders = get_placeholders_str(9, events.len());
    query_builder.push_str(&placeholders);
    query_builder.push_str(r#" ON CONFLICT (block_number, log_id) DO NOTHING"#);

    let query = events
        .into_iter()
        .fold(sqlx::query(&query_builder), |query, event| {
            query
                .bind(event.block_number as i32)
                .bind(event.log_id)
                .bind(event.collateral_asset)
                .bind(event.debt_asset)
                .bind(event.user)
                .bind(event.debt_to_cover)
                .bind(event.liquidated_collateral_amount)
                .bind(event.liquidator)
                .bind(event.receive_a_token)
        });

    let res = query.execute(pool).await?;
    debug!("Insert liquidation events into db result: {:#?}", res);

    Ok(())
}
