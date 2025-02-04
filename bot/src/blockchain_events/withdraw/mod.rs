use alloy::rpc::types::Log;
use eyre::{eyre, Error, OptionExt, Result};
use serde::{Deserialize, Serialize};
use sqlx::{query_as, types::BigDecimal};
use std::str::FromStr;
use tracing::debug;

use crate::{contracts::pool::Pool::Withdraw, shared::persistence::Address};

/**
 * reserve The address of the underlying asset being withdrawn
 * user The address initiating the withdrawal, owner of aTokens
 * to The address that will receive the underlying
 * amount The amount to be withdrawn
 */
#[derive(Debug, sqlx::FromRow, Serialize, Deserialize)]
pub struct WithdrawEventDto {
    pub block_number: u64,
    pub log_number: u64,
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
            .ok_or(eyre!("Log must contain block number"))?;
        let log_number = value
            .log_index
            .ok_or_eyre("Event must have an event log id")?;
        let address = value.inner.address.to_vec();
        let reserve = value.inner.reserve.to_vec();
        let user = value.inner.user.to_vec();
        let to = value.inner.to.to_vec();
        let amount = BigDecimal::from_str(&value.inner.amount.to_string())?;

        Ok(WithdrawEventDto {
            block_number,
            log_number,
            address,
            reserve,
            user,
            to,
            amount,
        })
    }
}
