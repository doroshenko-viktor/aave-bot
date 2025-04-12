use alloy::{primitives::Address, rpc::types::Log};
use eyre::{Error, Result};
use serde::{Deserialize, Serialize};
use sqlx::types::BigDecimal;
use std::str::FromStr;

use crate::{
    accounts::types::AssetAmount, contracts::pool::Pool::Withdraw,
    shared::persistence::AccountAddress, tokens::TokenAddress,
};

/**
 * reserve The address of the underlying asset being withdrawn
 * user The address initiating the withdrawal, owner of aTokens
 * to The address that will receive the underlying
 * amount The amount to be withdrawn
 */
#[derive(Debug, sqlx::FromRow, Serialize, Deserialize)]
pub struct WithdrawEventDto {
    pub address: Address,
    pub reserve: TokenAddress,
    pub user: AccountAddress,
    pub to: AccountAddress,
    pub amount: AssetAmount,
}

impl TryFrom<&Log<Withdraw>> for WithdrawEventDto {
    type Error = Error;

    fn try_from(value: &Log<Withdraw>) -> Result<Self, Self::Error> {
        let address = value.inner.address;
        let reserve = value.inner.reserve;
        let user = value.inner.user;
        let to = value.inner.to;
        let amount = value.inner.amount;

        Ok(WithdrawEventDto { address, reserve, user, to, amount })
    }
}
