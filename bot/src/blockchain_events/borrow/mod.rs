pub mod get;
pub mod save;
use std::str::FromStr;

use crate::{contracts::pool::Pool::Borrow, shared::persistence::PostgresPool};
use alloy::rpc::types::Log;
use eyre::{eyre, Error, OptionExt, Result};
use sqlx::types::BigDecimal;
use tracing::{debug, error, info, warn};

/**
 * @dev Emitted on borrow() and flashLoan() when debt needs to be opened
 * @param reserve The address of the underlying asset being borrowed
 * @param user The address of the user initiating the borrow(), receiving the funds on borrow() or just
 * initiator of the transaction on flashLoan()
 * @param onBehalfOf The address that will be getting the debt
 * @param amount The amount borrowed out
 * @param interestRateMode The rate mode: 2 for Variable, 1 is deprecated (changed on v3.2.0)
 * @param borrowRate The numeric rate at which the user has borrowed, expressed in ray
 * @param referralCode The referral code used
 */
#[derive(Debug, sqlx::FromRow)]
pub struct BorrowEventDto {
    pub block_number: i32,
    pub log_id: i32,
    pub amount: BigDecimal,
    pub borrow_rate: BigDecimal,
    pub interest_rate_mode: i32,
    pub on_behalf_of: Vec<u8>,
    pub referral_code: i32,
    pub reserve: Vec<u8>,
    pub user: Vec<u8>,
}

impl TryFrom<&Log<Borrow>> for BorrowEventDto {
    type Error = Error;

    fn try_from(value: &Log<Borrow>) -> Result<Self, Error> {
        let block_number = value
            .block_number
            .ok_or(eyre!("Log must contain block number"))? as i32;
        let log_id = value
            .log_index
            .ok_or_eyre("Event must have an event log id")? as i32;
        let amount = BigDecimal::from_str(&value.inner.amount.to_string())?;
        let borrow_rate = BigDecimal::from_str(&value.inner.borrowRate.to_string())?;
        let interest_rate_mode = value.inner.interestRateMode as i32;
        let on_behalf_of = value.inner.onBehalfOf.to_vec();
        let referral_code = value.inner.referralCode as i32;
        let reserve = value.inner.reserve.to_vec();
        let user = value.inner.user.to_vec();
        Ok(BorrowEventDto {
            block_number,
            log_id,
            amount,
            borrow_rate,
            interest_rate_mode,
            on_behalf_of,
            referral_code,
            reserve,
            user,
        })
    }
}
