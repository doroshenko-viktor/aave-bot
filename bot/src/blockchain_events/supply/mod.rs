pub mod get;
pub mod save;
use std::str::FromStr;

use alloy::rpc::types::Log;
use bigdecimal::BigDecimal;
use eyre::{Error, OptionExt};

use crate::contracts::pool::Pool::Supply;

/**
 * @dev Emitted on supply()
 * @param reserve The address of the underlying asset of the reserve
 * @param user The address initiating the supply
 * @param onBehalfOf The beneficiary of the supply, receiving the aTokens
 * @param amount The amount supplied
 * @param referralCode The referral code used
 */
#[derive(Debug, sqlx::FromRow)]
pub struct SupplyEventDto {
    pub block_number: i32,
    pub log_id: i32,
    pub reserve: Vec<u8>,
    pub user: Vec<u8>,
    pub on_behalf_of: Vec<u8>,
    pub amount: BigDecimal,
    pub referral_code: i32,
}

impl TryFrom<&Log<Supply>> for SupplyEventDto {
    type Error = Error;

    fn try_from(value: &Log<Supply>) -> Result<Self, Error> {
        let block_number = value
            .block_number
            .ok_or_eyre("Log must contain block number")? as i32;
        let log_id = value
            .log_index
            .ok_or_eyre("Event must have an event log id")? as i32;
        let reserve = value.inner.reserve.to_vec();
        let user = value.inner.user.to_vec();
        let on_behalf_of = value.inner.onBehalfOf.to_vec();
        let amount = BigDecimal::from_str(&value.inner.amount.to_string())?;
        let referral_code = value.inner.referralCode as i32;
        Ok(SupplyEventDto {
            block_number,
            log_id,
            amount,
            on_behalf_of,
            referral_code,
            reserve,
            user,
        })
    }
}
