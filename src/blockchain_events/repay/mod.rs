use std::str::FromStr;

use crate::{
    accounts::types::AssetAmount, contracts::pool::Pool::Repay,
    shared::persistence::AccountAddress, tokens::TokenAddress,
};

use alloy::rpc::types::Log;
use eyre::{Error, OptionExt, Result};
use serde::{Deserialize, Serialize};
use sqlx::types::BigDecimal;

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
    pub reserve: TokenAddress,
    pub user: AccountAddress,
    pub repayer: AccountAddress,
    pub amount: AssetAmount,
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
            reserve: value.data().reserve,
            user: value.data().user,
            repayer: value.data().repayer,
            amount: value.data().amount,
            use_a_tokens: value.data().useATokens,
        })
    }
}
