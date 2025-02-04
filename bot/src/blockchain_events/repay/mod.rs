use std::str::FromStr;

use crate::{
    contracts::pool::Pool::Repay,
    shared::persistence::{get_placeholders_str, Address, DbPool, SCHEMA_BLOCKCHAIN},
};

use alloy::rpc::types::Log;
use eyre::{Error, OptionExt, Result};
use serde::{Deserialize, Serialize};
use sqlx::{query_as, types::BigDecimal};

/**
 * reserve The address of the underlying asset of the reserve
 * user The beneficiary of the repayment, getting his debt reduced
 * repayer The address of the user initiating the repay(), providing the funds
 * amount The amount repaid
 * useATokens True if the repayment is done using aTokens, `false` if done with underlying asset directly
 */
#[derive(Debug, sqlx::FromRow, Serialize, Deserialize)]
pub struct RepayEventDto {
    pub block_number: u64,
    pub log_number: u64,
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
                .ok_or_eyre("Event must have block number")?,
            log_number: value.log_index.ok_or_eyre("Event must have log id")?,
            reserve: value.data().reserve.to_vec(),
            user: value.data().user.to_vec(),
            repayer: value.data().repayer.to_vec(),
            amount: BigDecimal::from_str(&value.data().amount.to_string())?,
            use_a_tokens: value.data().useATokens,
        })
    }
}
