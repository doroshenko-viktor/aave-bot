use std::str::FromStr;

use alloy::rpc::types::Log;
use async_trait::async_trait;
use eyre::{Error, OptionExt, Result};
use sqlx::{query_as, types::BigDecimal};
use tracing::debug;

use crate::{contracts::pool::Pool::LiquidationCall, persistence::SCHEMA_BLOCKCHAIN};

use super::{Address, PostgresPool};

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

pub async fn save_liquidation_events(
    pool: &PostgresPool,
    events: Vec<LiquidationEventDto>,
) -> Result<()> {
    debug!("Inserting {} liquidation events to db", events.len());
    let mut query_builder = format!(
        r#"INSERT INTO {}.liquidation_events (
                block_number,
                log_id,
                collateral_asset,
                debt_asset,
                "user",
                debt_to_cover,
                liquidated_collateral_amount,
                liquidator,
                receive_a_token
            ) VALUES"#,
        SCHEMA_BLOCKCHAIN
    );
    let placeholders = super::get_placeholders_str(9, events.len());
    query_builder.push_str(&placeholders);
    query_builder.push_str(r#" ON CONFLICT (block_number, log_id) DO NOTHING"#);

    let query = events
        .into_iter()
        .fold(sqlx::query(&query_builder), |query, event| {
            query
                .bind(event.block_number as i32)
                .bind(event.log_id)
                .bind(event.collateral_asset)
                .bind(event.debt_asset)
                .bind(event.user)
                .bind(event.debt_to_cover)
                .bind(event.liquidated_collateral_amount)
                .bind(event.liquidator)
                .bind(event.receive_a_token)
        });

    let res = query.execute(pool).await?;
    debug!("Insert liquidation events into db result: {:#?}", res);

    Ok(())
}

pub async fn get_liquidations_sorted_by_block(
    pool: &PostgresPool,
    from_block: i32,
    to_block: i32,
) -> Result<Vec<LiquidationEventDto>> {
    let events = query_as!(
        LiquidationEventDto,
        r#"
        select
            block_number,
            log_id,
            collateral_asset,
            debt_asset,
            "user",
            debt_to_cover,
            liquidated_collateral_amount,
            liquidator,
            receive_a_token
        from blockchain.liquidation_events
        where block_number >= $1 and block_number <= $2
        order by block_number asc;
        "#,
        from_block,
        to_block
    )
    .fetch_all(pool)
    .await?;

    Ok(events)
}
