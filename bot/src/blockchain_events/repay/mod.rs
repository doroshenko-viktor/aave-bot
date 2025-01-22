pub mod get;
pub mod save;

use std::str::FromStr;

use crate::{
    contracts::pool::Pool::Repay,
    shared::persistence::{get_placeholders_str, Address, PostgresPool, SCHEMA_BLOCKCHAIN},
};

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
