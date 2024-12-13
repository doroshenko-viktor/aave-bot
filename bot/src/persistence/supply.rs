use std::str::FromStr;

use alloy::rpc::types::Log;
use eyre::{eyre, Error, Ok, OptionExt, Result};
use futures::future::join_all;
use sqlx::{query, query_as, types::BigDecimal};
use tracing::{debug, info};

use crate::{
    contracts::pool::Pool::Supply,
    persistence::{get_placeholders_str, SCHEMA_BLOCKCHAIN},
};

use super::PostgresPool;

/**
 * @dev Emitted on supply()
 * @param reserve The address of the underlying asset of the reserve
 * @param user The address initiating the supply
 * @param onBehalfOf The beneficiary of the supply, receiving the aTokens
 * @param amount The amount supplied
 * @param referralCode The referral code used
 */
#[derive(Debug, sqlx::FromRow)]
pub struct SupplyEventDto {
    pub block_number: i32,
    pub log_id: i32,
    pub reserve: Vec<u8>,
    pub user: Vec<u8>,
    pub on_behalf_of: Vec<u8>,
    pub amount: BigDecimal,
    pub referral_code: i32,
}

impl TryFrom<&Log<Supply>> for SupplyEventDto {
    type Error = Error;

    fn try_from(value: &Log<Supply>) -> Result<Self, Error> {
        let block_number = value
            .block_number
            .ok_or(eyre!("Log must contain block number"))? as i32;
        let log_id = value
            .log_index
            .ok_or_eyre("Event must have an event log id")? as i32;
        let reserve = value.inner.reserve.to_vec();
        let user = value.inner.user.to_vec();
        let on_behalf_of = value.inner.onBehalfOf.to_vec();
        let amount = BigDecimal::from_str(&value.inner.amount.to_string())?;
        let referral_code = value.inner.referralCode as i32;
        Ok(SupplyEventDto {
            block_number,
            log_id,
            amount,
            on_behalf_of,
            referral_code,
            reserve,
            user,
        })
    }
}

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
