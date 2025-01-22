pub mod get;
pub mod save;

use crate::{
    app::entities::AccountEntity,
    shared::persistence::{get_placeholders_str, PostgresPool},
};
use bigdecimal::BigDecimal;
use eyre::Result;
use tracing::debug;

#[derive(Debug, sqlx::FromRow)]
pub struct AccountInsertDto {
    pub address: Vec<u8>,
}

#[derive(Debug, sqlx::FromRow)]
pub struct AccountCollateralDto {
    pub account_id: i32,
    pub asset_id: i32,
    pub amount: BigDecimal,
}

#[derive(Debug, sqlx::FromRow)]
pub struct AccountDebtDto {
    pub account_id: i32,
    pub asset_id: i32,
    pub amount: BigDecimal,
}
