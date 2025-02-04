use std::str::FromStr;

use alloy::rpc::types::Log;
use bigdecimal::BigDecimal;
use eyre::{Error, OptionExt};
use serde::{Deserialize, Serialize};

use crate::contracts::pool::Pool::Supply;

/**
 * @dev Emitted on supply()
 * @param reserve The address of the underlying asset of the reserve
 * @param user The address initiating the supply
 * @param onBehalfOf The beneficiary of the supply, receiving the aTokens
 * @param amount The amount supplied
 * @param referralCode The referral code used
 */
#[derive(Debug, sqlx::FromRow, Serialize, Deserialize)]
pub struct SupplyEventDto {
    pub block_number: u64,
    pub log_number: u64,
    pub reserve: Vec<u8>,
    pub user: Vec<u8>,
    pub on_behalf_of: Vec<u8>,
    pub amount: BigDecimal,
    pub referral_code: u16,
}

impl TryFrom<&Log<Supply>> for SupplyEventDto {
    type Error = Error;

    fn try_from(value: &Log<Supply>) -> Result<Self, Error> {
        let block_number = value
            .block_number
            .ok_or_eyre("Log must contain block number")?;
        let log_number = value
            .log_index
            .ok_or_eyre("Event must have an event log id")?;
        let reserve = value.inner.reserve.to_vec();
        let user = value.inner.user.to_vec();
        let on_behalf_of = value.inner.onBehalfOf.to_vec();
        let amount = BigDecimal::from_str(&value.inner.amount.to_string())?;
        let referral_code = value.inner.referralCode;
        Ok(SupplyEventDto {
            block_number,
            log_number,
            amount,
            on_behalf_of,
            referral_code,
            reserve,
            user,
        })
    }
}
