pub mod account_store;
pub mod config;
pub mod entities;
pub mod health_factor;
pub mod types;
pub mod value_objects;

use crate::shared::persistence::AccountAddress;

#[derive(Debug, sqlx::FromRow)]
pub struct AccountInsertDto {
    pub address: AccountAddress,
}

#[derive(Debug, sqlx::FromRow)]
pub struct AccountCollateralDto {
    pub account_address: String,
    pub token_symbol: String,
    pub amount: String,
}

#[derive(Debug, sqlx::FromRow)]
pub struct AccountDebtDto {
    pub account_address: String,
    pub token_symbol: String,
    pub amount: String,
}
