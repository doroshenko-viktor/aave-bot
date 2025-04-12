use dashmap::DashMap;
use std::sync::LazyLock;

use crate::{price_feed::entities::TokenPrice, tokens::types::TokenId};

/*
Contains last received prices of all relevant assets to `USD`.
This is a thread safe struct and may be safely cloned between threads.
*/
static USD_PRICES: LazyLock<DashMap<TokenId, TokenPrice>> =
    LazyLock::new(|| DashMap::default());

pub fn get_usd_price(token_id: TokenId) -> Option<TokenPrice> {
    USD_PRICES.get(&token_id).map(|val| *val)
}

pub fn set_usd_price(token_id: TokenId, price: TokenPrice) {
    if let Some(mut val) = USD_PRICES.get_mut(&token_id) {
        *val = price;
    } else {
        USD_PRICES.insert(token_id, price);
    }
}

//     let prices = self.prices.read().await;
//     let ticker_value = Ticker::get_usd_ticker_value(token);
//     prices.get(&ticker_value).cloned()
// }

// pub async fn get_assets_compound_usd_value(
//     &self,
//     assets: &[(TokenSymbol, AssetAmount)],
// ) -> Result<BigDecimal> {
//     let prices = self.prices.read().await;
//     let mut total_usd_value = BigDecimal::zero();
//     for (token_symbol, asset_amount) in assets {
//         let ticker_value = Ticker::get_usd_ticker_value(token_symbol);
//         let price = prices
//             .get(&ticker_value)
//             .ok_or_eyre("Price for ticker {} has not been found in prices aggregator")?;
//         let asset_usd_value = asset_amount * price;
//         total_usd_value += asset_usd_value;
//     }

//     Ok(total_usd_value)
// }
