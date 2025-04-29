use crate::{
    accounts::types::AssetAmount, contracts::pool::Pool::LiquidationCall,
    shared::persistence::AccountAddress, tokens::TokenAddress,
};
use alloy::rpc::types::Log;
use eyre::{Error, Result};
use serde::{Deserialize, Serialize};

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
#[derive(Debug, sqlx::FromRow, Serialize, Deserialize, Clone)]
pub struct LiquidationEventDto {
    pub collateral_asset: TokenAddress,
    pub debt_asset: TokenAddress,
    pub user: AccountAddress,
    pub debt_to_cover: AssetAmount,
    pub liquidated_collateral_amount: AssetAmount,
    pub liquidator: AccountAddress,
    pub receive_a_token: bool,
}

impl TryFrom<&Log<LiquidationCall>> for LiquidationEventDto {
    type Error = Error;

    fn try_from(value: &Log<LiquidationCall>) -> Result<Self, Self::Error> {
        Ok(LiquidationEventDto {
            collateral_asset: value.data().collateralAsset,
            debt_asset: value.data().debtAsset,
            user: value.data().user.into(),
            debt_to_cover: value.data().debtToCover,
            liquidated_collateral_amount: value
                .data()
                .liquidatedCollateralAmount,
            liquidator: value.data().liquidator.into(),
            receive_a_token: value.data().receiveAToken,
        })
    }
}
