use std::str::FromStr;

use super::{Address, PostgresPool};
use crate::{contracts::pool::Pool::Withdraw, persistence::SCHEMA_BLOCKCHAIN};
use alloy::rpc::types::Log;
use eyre::{eyre, Error, OptionExt, Result};
use sqlx::{query_as, types::BigDecimal};
use tracing::debug;

/**
 * reserve The address of the underlying asset being withdrawn
 * user The address initiating the withdrawal, owner of aTokens
 * to The address that will receive the underlying
 * amount The amount to be withdrawn
 */
#[derive(Debug, sqlx::FromRow)]
pub struct WithdrawEventDto {
    pub block_number: i32,
    pub log_id: i32,
    pub address: Address,
    pub reserve: Address,
    pub user: Address,
    pub to: Address,
    pub amount: BigDecimal,
}

impl TryFrom<&Log<Withdraw>> for WithdrawEventDto {
    type Error = Error;

    fn try_from(value: &Log<Withdraw>) -> Result<Self, Self::Error> {
        let block_number = value
            .block_number
            .ok_or(eyre!("Log must contain block number"))? as i32;
        let log_id = value
            .log_index
            .ok_or_eyre("Event must have an event log id")? as i32;
        let address = value.inner.address.to_vec();
        let reserve = value.inner.reserve.to_vec();
        let user = value.inner.user.to_vec();
        let to = value.inner.to.to_vec();
        let amount = BigDecimal::from_str(&value.inner.amount.to_string())?;

        Ok(WithdrawEventDto {
            block_number,
            log_id,
            address,
            reserve,
            user,
            to,
            amount,
        })
    }
}

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
    let placeholders = super::get_placeholders_str(7, events.len());
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

pub async fn get_withdraw_events(
    pool: &PostgresPool,
    from_block: i32,
    to_block: i32,
) -> Result<Vec<WithdrawEventDto>> {
    let events = query_as!(
        WithdrawEventDto,
        r#"
            select
                block_number,
                log_id,
                address,
                reserve,
                "user",
                "to",
                amount
            from blockchain.withdraw_events
            where block_number >= $1 and block_number <= $2
        "#,
        from_block,
        to_block
    )
    .fetch_all(pool)
    .await?;

    Ok(events)
}
