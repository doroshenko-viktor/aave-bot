use std::str::FromStr;

use crate::{
    contracts::pool::Pool::Borrow,
    persistence::{get_placeholders_str, SCHEMA_BLOCKCHAIN},
};
use alloy::rpc::types::Log;
use async_trait::async_trait;
use eyre::{eyre, Error, OptionExt};
use sqlx::{types::BigDecimal, Pool, Postgres};
use tracing::{debug, error, info, warn};

#[derive(Debug, sqlx::FromRow)]
pub struct BorrowEventDto {
    pub block_number: u64,
    pub log_id: i64,
    pub amount: BigDecimal,
    pub borrow_rate: BigDecimal,
    pub interest_rate_mode: i16,
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
            .ok_or(eyre!("Log must contain block number"))?;
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

#[async_trait]
pub trait BorrowEventsPersistence {
    async fn save_borrow_events(&self, events: Vec<BorrowEventDto>) -> Result<(), Error>;
}

#[async_trait]
impl BorrowEventsPersistence for Pool<Postgres> {
    async fn save_borrow_events(&self, events: Vec<BorrowEventDto>) -> Result<(), Error> {
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
        let res = query.execute(self).await?;
        info!("INSERTION RES: {:#?}", res);

        Ok(())
    }
}
