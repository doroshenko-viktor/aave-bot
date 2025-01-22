use eyre::{Ok, Result};
use futures::future::join_all;
use sqlx::query_as;
use tracing::{debug, info};

use crate::shared::persistence::{get_placeholders_str, PostgresPool, SCHEMA_BLOCKCHAIN};

use super::SupplyEventDto;

pub async fn get_supply_events(
    pool: &PostgresPool,
    from_block: i32,
    to_block: i32,
) -> Result<Vec<SupplyEventDto>> {
    let events = query_as!(
        SupplyEventDto,
        r#"
            select
                block_number,
                log_id,
                reserve,
                "user",
                on_behalf_of,
                amount,
                referral_code
            from blockchain.supply_events
            where block_number >= $1 and block_number <= $2
        "#,
        from_block,
        to_block
    )
    .fetch_all(pool)
    .await?;

    Ok(events)
}
