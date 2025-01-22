use super::RepayEventDto;
use crate::shared::persistence::{get_placeholders_str, PostgresPool, SCHEMA_BLOCKCHAIN};
use eyre::Result;
use tracing::debug;

pub async fn save_repay_events(pool: &PostgresPool, events: Vec<RepayEventDto>) -> Result<()> {
    debug!("Saving {} repay events", events.len());
    let mut query_builder = format!(
        r#"INSERT INTO {}.repay_events (
                block_number,
                log_id,
                amount,
                reserve,
                "user",
                repayer,
                use_a_tokens
            ) VALUES"#,
        SCHEMA_BLOCKCHAIN
    );
    let placeholders = get_placeholders_str(7, events.len());
    query_builder.push_str(&placeholders);
    query_builder.push_str(r#" ON CONFLICT (block_number, log_id) DO NOTHING"#);
    let mut query = sqlx::query(&query_builder);

    for event in events {
        query = query
            .bind(event.block_number as i32)
            .bind(event.log_id)
            .bind(event.amount)
            .bind(event.reserve)
            .bind(event.user)
            .bind(event.repayer)
            .bind(event.use_a_tokens);
    }
    let res = query.execute(pool).await?;
    debug!("Insert repay events result: {:#?}", res);

    Ok(())
}
