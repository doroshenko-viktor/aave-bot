use std::str::FromStr;

use crate::contracts::pool::Pool::Repay;
use crate::persistence::SCHEMA_BLOCKCHAIN;

use super::{Address, PostgresPool};
use alloy::rpc::types::Log;
use eyre::{Error, OptionExt, Result};
use sqlx::{query_as, types::BigDecimal};
use tracing::debug;

/**
 * reserve The address of the underlying asset of the reserve
 * user The beneficiary of the repayment, getting his debt reduced
 * repayer The address of the user initiating the repay(), providing the funds
 * amount The amount repaid
 * useATokens True if the repayment is done using aTokens, `false` if done with underlying asset directly
 */
#[derive(Debug, sqlx::FromRow)]
pub struct RepayEventDto {
    pub block_number: i32,
    pub log_id: i32,
    pub reserve: Address,
    pub user: Address,
    pub repayer: Address,
    pub amount: BigDecimal,
    pub use_a_tokens: bool,
}

impl TryFrom<&Log<Repay>> for RepayEventDto {
    type Error = Error;

    fn try_from(value: &Log<Repay>) -> Result<Self, Self::Error> {
        Ok(RepayEventDto {
            block_number: value
                .block_number
                .ok_or_eyre("Event must have block number")? as i32,
            log_id: value.log_index.ok_or_eyre("Event must have log id")? as i32,
            reserve: value.data().reserve.to_vec(),
            user: value.data().user.to_vec(),
            repayer: value.data().repayer.to_vec(),
            amount: BigDecimal::from_str(&value.data().amount.to_string())?,
            use_a_tokens: value.data().useATokens,
        })
    }
}

pub async fn save_repay_events(
    pool: &PostgresPool,
    events: Vec<RepayEventDto>,
) -> Result<()> {
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
    let placeholders = super::get_placeholders_str(7, events.len());
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

pub async fn get_repays_sorted_by_block(
    pool: &PostgresPool,
    from_block: i32,
    to_block: i32,
) -> Result<Vec<RepayEventDto>> {
    let events = query_as!(
        RepayEventDto,
        r#"
        select
            block_number,
            log_id,
            amount,
            reserve,
            "user",
            repayer,
            use_a_tokens
        from blockchain.repay_events
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
