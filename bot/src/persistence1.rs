use std::{
    collections::{binary_heap, HashMap, HashSet},
    hash::Hash,
    str::FromStr,
};

use alloy::rpc::types::{ExtAccountInfo, Log};
use eyre::{Error, OptionExt};
use sqlx::{
    query,
    types::{
        time::{OffsetDateTime, PrimitiveDateTime},
        BigDecimal,
    },
    Pool, Postgres,
};

use crate::{
    contracts::pool::{
        DataTypes::InterestRateMode,
        Pool::{Borrow, LiquidationCall, Repay, Withdraw},
    },
    utils,
};

pub async fn save_repay_event(pool: &Pool<Sqlite>, x: Log<Repay>) -> Result<(), Error> {
    let block_hash = x.block_hash.as_ref().map(|x| x.as_slice());
    let block_number = x.block_number.map(|x| x.to_string());
    let block_timestamp = x.block_timestamp.map(utils::convert_timestamp_to_string);
    let amount = x.inner.amount.to_string();
    let address = x.inner.address.to_string();
    let use_a_tokens = x.inner.useATokens;
    let reserve = x.inner.data.reserve.to_string();
    let amount_data = x.inner.data.amount.to_string();
    let user = x.inner.user.to_string();
    let repayer = x.inner.data.repayer.to_string();
    let is_removed = x.removed;

    print!("block_number: {:?}", block_number);
    print!("block_timestamp: {:?}", block_timestamp);

    query!(
        "INSERT INTO eth_events_repays (
            block_hash,
            block_number,
            block_timestamp,
            amount,
            address,
            use_a_tokens,
            reserve,
            user,
            repayer,
            is_removed
        ) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9, $10)",
        block_hash,
        block_number,
        block_timestamp,
        amount,
        address,
        use_a_tokens,
        reserve,
        user,
        repayer,
        is_removed
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn save_withdraw_event(pool: &Pool<Sqlite>, x: Log<Withdraw>) -> Result<(), Error> {
    let block_hash = x.block_hash.as_ref().map(|x| x.as_slice());
    let block_number = x.block_number.map(|x| x.to_string());
    let block_timestamp = x.block_timestamp.map(utils::convert_timestamp_to_string);
    let amount = x.inner.amount.to_string();
    let address = x.inner.address.to_string();
    let reserve = x.inner.data.reserve.to_string();
    let amount_data = x.inner.data.amount.to_string();
    let user = x.inner.user.to_string();
    let is_removed = x.removed;
    let to_address = x.inner.to.to_string();

    print!("block_number: {:?}", block_number);
    print!("block_timestamp: {:?}", block_timestamp);

    query!(
        "INSERT INTO eth_events_withdraws (
            block_hash,
            block_number,
            block_timestamp,
            amount,
            address,
            reserve,
            user,
            to_address,
            is_removed
        ) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9)",
        block_hash,
        block_number,
        block_timestamp,
        amount,
        address,
        reserve,
        user,
        to_address,
        is_removed
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn save_liquidation_events(
    pool: &Pool<Sqlite>,
    x: Log<LiquidationCall>,
) -> Result<(), Error> {
    let block_hash = x.block_hash.as_ref().map(|x| x.as_slice());
    let block_number = x.block_number.map(|x| x.to_string());
    let block_timestamp = x.block_timestamp.map(utils::convert_timestamp_to_string);
    let address = x.inner.address.to_string();
    let user = x.inner.user.to_string();
    let is_removed = x.removed;
    let collateral_asset = x.inner.collateralAsset.to_string();
    let debt_asset = x.inner.debtAsset.to_string();
    let debt_to_cover = x.inner.debtToCover.to_string();
    let liquidated_collateral_amount = x.inner.liquidatedCollateralAmount.to_string();
    let liquidator = x.inner.liquidator.to_string();

    print!("block_number: {:?}", block_number);
    print!("block_timestamp: {:?}", block_timestamp);

    query!(
        "INSERT INTO eth_events_liquidations (
            block_hash ,
            block_number ,
            block_timestamp ,
            address ,
            collateral_asset ,
            debt_asset ,
            debt_to_cover ,
            liquidated_collateral_amount ,
            liquidator ,
            is_removed ,
            user 
        ) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9, $10, $11)",
        block_hash,
        block_number,
        block_timestamp,
        address,
        collateral_asset,
        debt_asset,
        debt_to_cover,
        liquidated_collateral_amount,
        liquidator,
        is_removed,
        user
    )
    .execute(pool)
    .await?;

    Ok(())
}
