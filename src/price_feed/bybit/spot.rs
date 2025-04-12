use bigdecimal::BigDecimal;
use eyre::{OptionExt, Result, eyre};
use serde_json::Value;
use std::str::FromStr;
use tracing::{debug, error, info};

use crate::price_feed::{
    ChangedTokenPriceTX, prices_aggregator, ticker_service,
};

use super::BybitEventRX;

pub fn listen(
    mut bybit_rx: BybitEventRX,
    changed_token_price_tx: ChangedTokenPriceTX,
) {
    tokio::spawn(async move {
        info!("Listening for bybit responses");
        while let Some(response) = bybit_rx.recv().await {
            match response {
                Ok(message) => {
                    match serde_json::from_slice::<Value>(message.as_bytes()) {
                        Ok(value) => {
                            handle_topic(value, changed_token_price_tx.clone())
                                .await;
                        }
                        Err(_) => todo!(),
                    }
                }
                Err(e) => error!(
                    "Error from bybit ws client stream: {}",
                    e
                ),
            }
        }
    });
}

//TODO: to not parse whole json:
// fn extract_price(json_str: &str) -> Option<f64> {
//     let mut deserializer = Deserializer::from_str(json_str).into_iter();

//     while let Some(Ok(token)) = deserializer.next() {
//         if token == Token::String && deserializer.get_ref().as_bytes().starts_with(b"price") {
//             deserializer.next();
//             if let Some(Ok(Token::String(price_str))) = deserializer.next() {
//                 if let Ok(price) = price_str.parse::<f64>() {
//                     return Some(price);
//                 }
//             }
//         }
//     }
//     None
// }
async fn handle_topic(
    value: Value,
    changed_token_price_tx: ChangedTokenPriceTX,
) {
    match value.get("topic") {
        Some(serde_json::Value::String(topic)) => {
            if topic.len() > 8 && &topic[..7] == "tickers" {
                handle_ticker_feed(&value, topic, changed_token_price_tx).await;
            }
        }
        _ => error!("NO TOPIC: {}", value),
    }
}

async fn handle_ticker_feed<'a>(
    value: &'a Value,
    topic: &'a str,
    changed_token_price_tx: ChangedTokenPriceTX,
) -> Result<()> {
    let token = ticker_service::get_main_token_from_bybit_str(&topic[8..]);
    let (last_traded_price, usd_price) = value
        .get("data")
        .map(|data| {
            let usd_index_price: Result<f64> =
                get_value(&data, "usdIndexPrice");
            let last_price: Result<f64> = get_value(&data, "lastPrice");
            (usd_index_price, last_price)
        })
        .ok_or_eyre("No 'data' object found in ticker stream object")?;
    let last_traded_price = last_traded_price?;
    let usd_price = usd_price?;

    #[cfg(debug_assertions)]
    {
        let (_, ticker_data) =
            ticker_service::get_ticker_from_bybit_str(&topic[8..]);
        debug!(
            "TICKER: {}; LAST PRICE: {}; USD PRICE: {}",
            ticker_data.bybit_repr, last_traded_price, usd_price
        );
    }

    prices_aggregator::set_usd_price(token.get_id(), last_traded_price);

    changed_token_price_tx.send(token).await;

    Ok(())
}

fn get_value<T>(value: &Value, name: &str) -> Result<T>
where
    T: FromStr,
    <T as FromStr>::Err: std::error::Error + Send + Sync + 'static,
{
    Ok(
        match value.get(name).ok_or_eyre(format!(
            "Price feed should contain {}",
            name
        ))? {
            Value::String(extracted) => extracted.parse::<T>()?,
            _ => Err(eyre!("Value is not in string format"))?,
        },
    )
}
