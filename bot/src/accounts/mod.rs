pub mod data_store;
pub mod entities;
pub mod get;
pub mod save;
pub mod types;
pub mod value_objects;

use crate::{
    app::entities::AccountEntity,
    shared::persistence::{get_placeholders_str, DbPool},
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
