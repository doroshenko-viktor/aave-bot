mod get;
mod save;
use crate::{
    accounts::types::AssetAmount, shared::persistence::AccountAddress,
    tokens::entities::Token,
};
use alloy::primitives::TxHash;
use eyre::Result;
pub use get::get_movements_paginated;
pub use save::save_aave_account_movement;
use serde::{Deserialize, Serialize};
use sqlx::types::BigDecimal;
use std::fmt::Display;

#[derive(Debug, PartialEq, Eq, sqlx::Type, Serialize, Deserialize)]
#[sqlx(type_name = "aave_event_type")]
pub enum AaveEventType {
    #[sqlx(rename = "supply")]
    Supply,
    #[sqlx(rename = "withdraw")]
    Withdraw,
    #[sqlx(rename = "borrow")]
    Borrow,
    #[sqlx(rename = "repay")]
    Repay,
    #[sqlx(rename = "collateral_transfer")]
    CollateralTransfer,
    #[sqlx(rename = "liquidation_repay_debt")]
    LiquidationRepayDebt,
    #[sqlx(rename = "liquidation_withdraw_collateral")]
    LiquidationWithdrawCollateral,
}

// In database
// 'supply',
// 'withdraw',
// 'borrow',
// 'repay',
// 'collateral_transfer',
// 'liquidation_repay_debt',
// 'liquidation_withdraw_collateral',
impl AaveEventType {
    fn into_i32(&self) -> i32 {
        match self {
            AaveEventType::Supply => 0,
            AaveEventType::Withdraw => 1,
            AaveEventType::Borrow => 2,
            AaveEventType::Repay => 3,
            AaveEventType::CollateralTransfer => 4,
            AaveEventType::LiquidationRepayDebt => 5,
            AaveEventType::LiquidationWithdrawCollateral => 6,
        }
    }
}

impl Display for AaveEventType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let repr = match self {
            AaveEventType::Supply => "supply".to_string(),
            AaveEventType::Withdraw => "withdraw".to_string(),
            AaveEventType::Borrow => "borrow".to_string(),
            AaveEventType::Repay => "repay".to_string(),
            AaveEventType::LiquidationRepayDebt => {
                "liquidation_repay_debt".to_string()
            }
            AaveEventType::LiquidationWithdrawCollateral => {
                "liquidation_withdraw_collateral".to_string()
            }
            AaveEventType::CollateralTransfer => {
                "collateral_transfer".to_string()
            }
        };
        write!(f, "{}", repr)
    }
}

#[derive(Debug)]
pub struct AaveAccountMovement {
    pub block_number: u64,
    pub transaction_hash: TxHash,
    pub event_type: AaveEventType,
    pub account_address: AccountAddress,
    pub token: &'static Token,
    pub native_token_amount: AssetAmount,
    pub reserve_token_amount: AssetAmount,
}

#[derive(Debug, sqlx::FromRow)]
pub struct AaveAccountMovementDto {
    pub id: i64,
    pub transaction_hash: String,
    pub event_type: AaveEventType,
    pub account_address: String,
    pub token_id: i32,
    pub native_token_amount: BigDecimal,
    pub reserve_token_amount: BigDecimal,
}

impl AaveAccountMovement {
    pub fn new(
        block_number: u64,
        transaction_hash: TxHash,
        event_type: AaveEventType,
        account_address: AccountAddress,
        token: &'static Token,
        native_token_amount: AssetAmount,
        reserve_token_amount: AssetAmount,
    ) -> Self {
        Self {
            block_number,
            transaction_hash,
            event_type,
            account_address,
            token,
            native_token_amount,
            reserve_token_amount,
        }
    }

    pub fn get_native_normalized_amount(&self) -> Result<String> {
        self.token.get_repr(self.native_token_amount)
    }
    pub fn get_reserve_normalized_amount(&self) -> Result<String> {
        self.token.get_repr(self.reserve_token_amount)
    }
}
