use alloy::rpc::types::Log;
use eyre::Error;
use sqlx::{query, Pool, Sqlite};

use crate::{
    contracts::pool::Pool::{Borrow, LiquidationCall, Repay, Withdraw},
    utils,
};

pub async fn save_borrow_event(pool: &Pool<Sqlite>, x: Log<Borrow>) -> Result<(), Error> {
    let block_hash = x.block_hash.as_ref().map(|x| x.as_slice());
    let block_number = x.block_number.map(|x| x.to_string());
    let block_timestamp = x.block_timestamp.map(utils::convert_timestamp_to_string);
    let amount = x.inner.amount.as_le_slice();
    let borrow_rate = x.inner.borrowRate.as_le_slice();
    let on_behalf_of = x.inner.onBehalfOf.as_slice();
    let referral_code = x.inner.referralCode.to_string();
    let reserve = x.inner.reserve.as_slice();
    let user = x.inner.user.as_slice();

    print!("block_number: {:?}", block_number);
    print!("block_timestamp: {:?}", block_timestamp);

    query!(
        "INSERT INTO eth_events_borrows (
            block_hash,
            block_number,
            block_timestamp,
            amount,
            borrow_rate,
            on_behalf_of,
            referral_code,
            reserve,
            user
        ) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9)",
        block_hash,
        block_number,
        block_timestamp,
        amount,
        borrow_rate,
        on_behalf_of,
        referral_code,
        reserve,
        user
    )
    .execute(pool)
    .await?;

    Ok(())
}

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
