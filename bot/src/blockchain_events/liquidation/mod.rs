pub mod get;
pub mod save;

use std::str::FromStr;

use alloy::rpc::types::Log;
use eyre::{Error, OptionExt, Result};
use sqlx::{query_as, types::BigDecimal};
use tracing::debug;

use crate::{contracts::pool::Pool::LiquidationCall, shared::persistence::Address};

/**
 * collateralAsset The address of the underlying asset used as collateral, to receive as result of the liquidation
 * debtAsset The address of the underlying borrowed asset to be repaid with the liquidation
 * user The address of the borrower getting liquidated
 * debtToCover The debt amount of borrowed `asset` the liquidator wants to cover
 * liquidatedCollateralAmount The amount of collateral received by the liquidator
 * liquidator The address of the liquidator
 * receiveAToken True if the liquidators wants to receive the collateral aTokens, `false` if he wants
 * to receive the underlying collateral asset directly
 */
#[derive(Debug, sqlx::FromRow)]
pub struct LiquidationEventDto {
    pub block_number: i32,
    pub log_id: i32,
    pub collateral_asset: Address,
    pub debt_asset: Address,
    pub user: Address,
    pub debt_to_cover: BigDecimal,
    pub liquidated_collateral_amount: BigDecimal,
    pub liquidator: Address,
    pub receive_a_token: bool,
}

impl TryFrom<&Log<LiquidationCall>> for LiquidationEventDto {
    type Error = Error;

    fn try_from(value: &Log<LiquidationCall>) -> Result<Self, Self::Error> {
        Ok(LiquidationEventDto {
            block_number: value
                .block_number
                .ok_or_eyre("Event must have block number")? as i32,
            log_id: value.log_index.ok_or_eyre("Event must have log id")? as i32,
            collateral_asset: value.data().collateralAsset.to_vec(),
            debt_asset: value.data().debtAsset.to_vec(),
            user: value.data().user.to_vec(),
            debt_to_cover: BigDecimal::from_str(&value.data().debtToCover.to_string())?,
            liquidated_collateral_amount: BigDecimal::from_str(
                &value.data().liquidatedCollateralAmount.to_string(),
            )?,
            liquidator: value.data().liquidator.to_vec(),
            receive_a_token: value.data().receiveAToken,
        })
    }
}
