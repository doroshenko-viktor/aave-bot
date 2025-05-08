mod account_store;
mod config;
mod entities;
mod health_factor;
pub mod types;
mod value_objects;
use crate::shared::persistence::AccountAddress;
pub use account_store::{
    UserAccountStore, backup_user_data_store, restore_user_account_store,
};
pub use config::AccountsConfig;
pub use entities::{UserAccount, UserAccountRef};
use sqlx::types::BigDecimal;

#[derive(Debug, sqlx::FromRow)]
pub struct AccountInsertDto {
    pub address: AccountAddress,
}

#[derive(Debug, sqlx::FromRow)]
pub struct AccountCollateralDto {
    pub account_address: String,
    pub token_symbol: String,
    pub amount: BigDecimal,
    pub a_token_amount: BigDecimal,
}

#[derive(Debug, sqlx::FromRow)]
pub struct AccountDebtDto {
    pub account_address: String,
    pub token_symbol: String,
    pub amount: BigDecimal,
    pub debt_token_amount: BigDecimal,
}
