use alloy::rpc::types::Log;
use eyre::Error;
use serde::{Deserialize, Serialize};

use crate::{
    accounts::types::AssetAmount, contracts::pool::Pool::Supply,
    shared::persistence::AccountAddress, tokens::TokenAddress,
};

/**
 * @dev Emitted on supply()
 * @param `reserve` The address of the underlying asset of the reserve
 * @param `user` The address initiating the supply
 * @param `onBehalfOf` The beneficiary of the supply, receiving the aTokens
 * @param `amount` The amount supplied
 * @param `referralCode` The referral code used
 */
#[derive(Debug, sqlx::FromRow, Serialize, Deserialize, Clone)]
pub struct SupplyEventDto {
    pub reserve: TokenAddress,
    pub user: AccountAddress,
    pub on_behalf_of: AccountAddress,
    pub amount: AssetAmount,
    pub referral_code: u16,
}

impl SupplyEventDto {
    pub fn on_behalf_of_same_account(&self) -> bool {
        self.user != self.on_behalf_of
    }
}

impl TryFrom<&Log<Supply>> for SupplyEventDto {
    type Error = Error;

    fn try_from(value: &Log<Supply>) -> Result<Self, Error> {
        let reserve = value.inner.reserve;
        let user = value.inner.user.into();
        let on_behalf_of = value.inner.onBehalfOf.into();
        let amount = value.inner.amount;
        let referral_code = value.inner.referralCode;

        Ok(SupplyEventDto {
            amount,
            on_behalf_of,
            referral_code,
            reserve,
            user,
        })
    }
}
