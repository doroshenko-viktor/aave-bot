use alloy::rpc::types::Log;
use eyre::{Error, Result};
use serde::{Deserialize, Serialize};

use crate::{
    accounts::types::AssetAmount, shared::persistence::AccountAddress,
    tokens::TokenAddress,
};

use super::contracts::IERC20::Transfer;

/**
 * This event is emitted when a transfer of AAVE AToken occurs.
 * It is emitted by the Aave a_token contract.
 * NOTE: This is the address of the aave a_token contract
 */
#[derive(Debug, sqlx::FromRow, Serialize, Deserialize, Clone)]
pub struct Erc20TransferEventDto {
    pub token_address: TokenAddress,
    pub from: AccountAddress,
    pub to: AccountAddress,
    pub value: AssetAmount,
}

impl TryFrom<&Log<Transfer>> for Erc20TransferEventDto {
    type Error = Error;

    fn try_from(value: &Log<Transfer>) -> Result<Self, Self::Error> {
        let token_address = value.address();
        let from = value.data().from.into();
        let to = value.data().to.into();
        let value = value.data().value;

        Ok(Erc20TransferEventDto { token_address, from, to, value })
    }
}
