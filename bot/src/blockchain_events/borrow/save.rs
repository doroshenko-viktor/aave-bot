use crate::{
    contracts::pool::Pool::Borrow,
    shared::persistence::{get_placeholders_str, PostgresPool, SCHEMA_BLOCKCHAIN},
};
use alloy::rpc::types::Log;
use eyre::{eyre, Error, OptionExt, Result};
use sqlx::{query, query_as, types::BigDecimal, Pool, Postgres};
use tracing::{debug, error, info, warn};

use super::BorrowEventDto;

pub async fn save_borrow_events(pool: &PostgresPool, events: Vec<BorrowEventDto>) -> Result<()> {
    debug!("Saving {} of borrow events to database", events.len());
    let mut query_builder = format!(
        r#"INSERT INTO {}.borrow_events (
                block_number,
                log_id,
                amount,
                borrow_rate,
                interest_rate_mode,
                on_behalf_of,
                referral_code,
                reserve,
                "user"
            ) VALUES"#,
        SCHEMA_BLOCKCHAIN
    );
    query_builder.push_str(&get_placeholders_str(9, events.len()));
    query_builder.push_str(r#" ON CONFLICT (block_number, log_id) DO NOTHING"#);
    let mut query = sqlx::query(&query_builder);

    for event in events {
        query = query
            .bind(event.block_number as i32)
            .bind(event.log_id)
            .bind(event.amount)
            .bind(event.borrow_rate)
            .bind(event.interest_rate_mode)
            .bind(event.on_behalf_of)
            .bind(event.referral_code)
            .bind(event.reserve)
            .bind(event.user);
    }
    let res = query.execute(pool).await?;
    info!("INSERTION RES: {:#?}", res);

    Ok(())
}
