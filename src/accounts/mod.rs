pub mod config;
pub mod entities;
pub mod health_factor;
pub mod user_account_store;
pub mod value_objects;

use bigdecimal::BigDecimal;
pub use user_account_store::backup_user_data_store;
pub use user_account_store::restore_user_account_store;

use crate::shared::persistence::AccountAddress;

#[derive(Debug, sqlx::FromRow)]
pub struct AccountInsertDto {
    pub address: AccountAddress,
}

#[derive(Debug, sqlx::FromRow)]
pub struct AccountCollateralDto {
    pub account_address: String,
    pub token_symbol: String,
    pub amount: BigDecimal,
}

#[derive(Debug, sqlx::FromRow)]
pub struct AccountDebtDto {
    pub account_address: String,
    pub token_symbol: String,
    pub amount: BigDecimal,
}
