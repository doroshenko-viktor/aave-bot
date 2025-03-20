use super::{entities::UserAccountRef, value_objects::AssetAmount};
use crate::{
    configuration::globals::get_asset_rules, price_feed::prices_aggregator, tokens::entities::Token,
};
use bigdecimal::{BigDecimal, Zero};
use eyre::{Result, eyre};
use futures::future::join_all;

/**
* Health Factor= ∑(Collateral Amount×Collateral Price×Liquidation Threshold)/ ∑(Borrowed Amount×Borrowed Asset Price)
*/
pub async fn calculate_health_factor(user_account: &UserAccountRef) -> Result<BigDecimal> {
    let user_account = user_account.read().await;
    Ok(get_asset_sum(user_account.get_collateral_list()).await?
        / get_asset_sum(user_account.get_debt_list()).await?)
}

async fn get_asset_sum(list: Vec<(&'static Token, AssetAmount)>) -> Result<BigDecimal> {
    let asset_rules = get_asset_rules();
    let collaterals = join_all(list.into_iter().map(|(token, amount)| async move {
        // TODO: asset rules = change to static service
        let threshold = asset_rules.get_asset_liquidation_threshold(token).await;
        // TODO: rebuild this func
        let price = prices_aggregator::get(token).await;
        if let Some(price) = price {
            return Ok((price, amount, threshold));
        }
        Err(eyre!(
            "Price for {} has not been found in price aggregator",
            token
        ))
    }))
    .await;
    let mut x = BigDecimal::zero();
    for collateral in collaterals {
        let (price, amount, threshold) = collateral?;
        x += price * amount * threshold;
    }

    Ok(x)
}
