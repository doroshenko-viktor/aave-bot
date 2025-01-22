use eyre::{Ok, Result};
use futures::future::join_all;
use tracing::{debug, info};

use crate::{
    blockchain_events::supply::SupplyEventDto,
    shared::persistence::{get_placeholders_str, PostgresPool, SCHEMA_BLOCKCHAIN},
};

pub async fn save_supply_events(pool: &PostgresPool, events: Vec<SupplyEventDto>) -> Result<()> {
    debug!("Saving {} of borrow events to database", events.len());
    let mut iter = events.into_iter();
    let mut events = Vec::new();

    while iter.by_ref().len() > 0 {
        let chunk: Vec<SupplyEventDto> = iter.by_ref().take(100).collect();
        events.push(chunk);
    }

    let queries = events.into_iter().map(|events| async move {
        let mut query_builder = format!(
            r#"INSERT INTO {}.supply_events (
                block_number,
                log_id,
                reserve,
                "user",
                on_behalf_of,
                amount,
                referral_code
            ) VALUES"#,
            SCHEMA_BLOCKCHAIN
        );
        query_builder.push_str(&get_placeholders_str(7, events.len()));
        query_builder.push_str(r#" ON CONFLICT (block_number, log_id) DO NOTHING"#);
        let query = events
            .into_iter()
            .fold(sqlx::query(&query_builder), |query, event| {
                query
                    .bind(event.block_number)
                    .bind(event.log_id)
                    .bind(event.reserve)
                    .bind(event.user)
                    .bind(event.on_behalf_of)
                    .bind(event.amount)
                    .bind(event.referral_code)
            });

        query.execute(pool).await
    });

    let results = join_all(queries).await;
    info!("INSERTION RESULTS:");
    for res in results {
        info!("{:#?}", res);
    }
    Ok(())
}
