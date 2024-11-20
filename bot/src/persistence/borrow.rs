use std::{collections::HashMap, str::FromStr};

use alloy::rpc::types::Log;
use eyre::{Error, OptionExt};
use sqlx::{types::BigDecimal, Pool, Postgres};

use crate::contracts::pool::Pool::Borrow;

use super::blocks::{BlockDbId, BlockDto};

#[derive(Debug, sqlx::FromRow)]
pub struct BorrowEventDto {
    pub block_number: u64,
    log_id: i64,
    amount: BigDecimal,
    borrow_rate: BigDecimal,
    interest_rate_mode: i16,
    on_behalf_of: Vec<u8>,
    referral_code: i32,
    reserve: Vec<u8>,
    user: Vec<u8>,
}

impl TryFrom<&Log<Borrow>> for BorrowEventDto {
    type Error = Error;

    fn try_from(value: &Log<Borrow>) -> Result<Self, Error> {
        let log_id = value
            .log_index
            .ok_or_eyre("Event must have an event log id")? as i64;
        let amount = BigDecimal::from_str(&value.inner.amount.to_string())?;
        let borrow_rate = BigDecimal::from_str(&value.inner.borrowRate.to_string())?;
        let interest_rate_mode = value.inner.interestRateMode as i16;
        let on_behalf_of = value.inner.onBehalfOf.to_vec();
        let referral_code = value.inner.referralCode as i32;
        let reserve = value.inner.reserve.to_vec();
        let user = value.inner.user.to_vec();
        Ok(BorrowEventDto {
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

pub async fn save_borrow_events(
    pool: &Pool<Postgres>,
    events: Vec<(BlockDbId, BorrowEventDto)>,
) -> Result<(), Error> {
    let mut query_builder = String::from(
        r#"INSERT INTO eth.borrow_events (
        block_id,
        log_id,
        amount,
        borrow_rate,
        interest_rate_mode,
        on_behalf_of,
        referral_code,
        reserve,
        "user"
    ) VALUES"#,
    );
    let mut ind = 1;
    let placeholders_amount = events.len() * 3;
    let mut placeholders = Vec::with_capacity(placeholders_amount as usize);
    while ind < placeholders_amount {
        placeholders.push(format!(
            "(${0}, ${1}, ${2}, ${3}, ${4}, ${5}, ${6}, ${7}, ${8})",
            ind,
            ind + 1,
            ind + 2,
            ind + 3,
            ind + 4,
            ind + 5,
            ind + 6,
            ind + 7,
            ind + 8
        ));
        ind += 9;
        query_builder.push_str(&placeholders.join(", "));
    }
    query_builder.push_str(r#" ON CONFLICT (block_id, log_id) DO NOTHING"#);
    let mut query = sqlx::query(&query_builder);

    for event in events {
        query = query
            .bind(event.block_id)
            .bind(event.log_id)
            .bind(event.amount)
            .bind(event.borrow_rate)
            .bind(event.interest_rate_mode)
            .bind(event.on_behalf_of)
            .bind(event.referral_code)
            .bind(event.reserve)
            .bind(event.user);
    }
    query.execute(pool).await;

    Ok(())
}
