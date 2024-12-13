use std::str::FromStr;

use crate::{
    contracts::pool::Pool::Borrow,
    persistence::{get_placeholders_str, SCHEMA_BLOCKCHAIN},
};
use alloy::rpc::types::Log;
use async_trait::async_trait;
use eyre::{eyre, Error, OptionExt, Result};
use sqlx::{query, query_as, types::BigDecimal, Pool, Postgres};
use tracing::{debug, error, info, warn};

use super::PostgresPool;

/**
 * @dev Emitted on borrow() and flashLoan() when debt needs to be opened
 * @param reserve The address of the underlying asset being borrowed
 * @param user The address of the user initiating the borrow(), receiving the funds on borrow() or just
 * initiator of the transaction on flashLoan()
 * @param onBehalfOf The address that will be getting the debt
 * @param amount The amount borrowed out
 * @param interestRateMode The rate mode: 2 for Variable, 1 is deprecated (changed on v3.2.0)
 * @param borrowRate The numeric rate at which the user has borrowed, expressed in ray
 * @param referralCode The referral code used
 */
#[derive(Debug, sqlx::FromRow)]
pub struct BorrowEventDto {
    pub block_number: i32,
    pub log_id: i32,
    pub amount: BigDecimal,
    pub borrow_rate: BigDecimal,
    pub interest_rate_mode: i32,
    pub on_behalf_of: Vec<u8>,
    pub referral_code: i32,
    pub reserve: Vec<u8>,
    pub user: Vec<u8>,
}

impl TryFrom<&Log<Borrow>> for BorrowEventDto {
    type Error = Error;

    fn try_from(value: &Log<Borrow>) -> Result<Self, Error> {
        let block_number = value
            .block_number
            .ok_or(eyre!("Log must contain block number"))? as i32;
        let log_id = value
            .log_index
            .ok_or_eyre("Event must have an event log id")? as i32;
        let amount = BigDecimal::from_str(&value.inner.amount.to_string())?;
        let borrow_rate = BigDecimal::from_str(&value.inner.borrowRate.to_string())?;
        let interest_rate_mode = value.inner.interestRateMode as i32;
        let on_behalf_of = value.inner.onBehalfOf.to_vec();
        let referral_code = value.inner.referralCode as i32;
        let reserve = value.inner.reserve.to_vec();
        let user = value.inner.user.to_vec();
        Ok(BorrowEventDto {
            block_number,
            log_id,
            amount,
            borrow_rate,
            interest_rate_mode,
            on_behalf_of,
            referral_code,
            reserve,
            user,
        })
    }
}

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

pub async fn get_borrows_sorted_by_block(
    pool: &PostgresPool,
    from_block: i32,
    to_block: i32,
) -> Result<Vec<BorrowEventDto>> {
    let events = query_as!(
        BorrowEventDto,
        r#"
        select
            block_number,
            log_id,
            amount,
            borrow_rate,
            interest_rate_mode,
            on_behalf_of,
            referral_code,
            reserve,
            "user"
        from blockchain.borrow_events
        where block_number >= $1 and block_number <= $2
        order by block_number asc;
        "#,
        from_block,
        to_block
    )
    .fetch_all(pool)
    .await?;

    Ok(events)
}
