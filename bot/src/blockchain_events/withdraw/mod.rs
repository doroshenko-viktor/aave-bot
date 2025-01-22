pub mod get;
pub mod save;

use std::str::FromStr;

use alloy::rpc::types::Log;
use eyre::{eyre, Error, OptionExt, Result};
use sqlx::{query_as, types::BigDecimal};
use tracing::debug;

use crate::{contracts::pool::Pool::Withdraw, shared::persistence::Address};

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
