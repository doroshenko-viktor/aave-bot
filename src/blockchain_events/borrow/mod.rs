use crate::{accounts::types::AssetAmount, contracts::pool::Pool::Borrow};
use alloy::{
    primitives::{Address, Uint},
    rpc::types::Log,
};
use eyre::{Error, Result};
use serde::{Deserialize, Serialize};
use sqlx::types::BigDecimal;
use std::str::FromStr;

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
#[derive(Debug, sqlx::FromRow, Serialize, Deserialize)]
pub struct BorrowEventDto {
    pub amount: AssetAmount,
    pub borrow_rate: Uint<256, 4>,
    pub interest_rate_mode: u8,
    pub on_behalf_of: Address,
    pub referral_code: u16,
    pub reserve: Address,
    pub user: Address,
}

impl TryFrom<&Log<Borrow>> for BorrowEventDto {
    type Error = Error;

    fn try_from(value: &Log<Borrow>) -> Result<Self, Error> {
        let amount = value.inner.amount;
        let borrow_rate = value.inner.borrowRate;
        let interest_rate_mode = value.inner.interestRateMode;
        let on_behalf_of = value.inner.onBehalfOf;
        let referral_code = value.inner.referralCode;
        let reserve = value.inner.reserve;
        let user = value.inner.user;
        Ok(BorrowEventDto {
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
