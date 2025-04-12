use eyre::Result;

#[tokio::main]
pub async fn main() -> Result<()> {
    Ok(())
    // let res = setup::configure_app().await?;
    // let bybit_spot_url = "wss://stream-testnet.bybit.com/v5/public/spot".to_string();
    // info!("URL: {bybit_spot_url}");
    //     let (mut conn, mut rx) = run_bybit_ws_channel::<BybitClientSpot>(bybit_spot_url).await?;

    //     let ticker = Ticker::from_str("BTCUSDT")?;

    //     info!("subscribing to {}", ticker.bybit());
    //     conn.subscribe_price(ticker).await;

    //     info!("Listening for bybit responses");
    //     while let Some(response) = rx.recv().await {
    //         match response {
    //             Ok(message) => {
    //                 // info!("{}", message);
    //                 match serde_json::from_str::<Value>(&message) {
    //                     Ok(value) => {
    //                         handle_topic(value).await;
    //                     }
    //                     Err(_) => todo!(),
    //                 }
    //             }
    //             Err(e) => error!("Error from bybit ws client stream: {}", e),
    //         }
    //     }

    //     warn!("Finished listening for bybit prices");
    //     Ok(())
    // }

    // async fn handle_topic(value: Value) {
    //     match value.get("topic") {
    //         Some(serde_json::Value::String(topic)) => {
    //             // debug!("TOPIC: {}", value);
    //             if topic.len() > 8 && &topic[..7] == "tickers" {
    //                 let ticker = &topic[8..];
    //                 let (usd_price, last_traded_price) = match value.get("data") {
    //                     Some(data) => {
    //                         let x: f64 = match data.get("usdIndexPrice") {
    //                             Some(Value::String(usd_price)) => usd_price.parse().unwrap(),
    //                             _ => todo!(),
    //                         };
    //                         let last_price: f64 = match data.get("lastPrice") {
    //                             Some(Value::String(last_price)) => last_price.parse().unwrap(),
    //                             _ => {
    //                                 error!("Can't parse last traded price");
    //                                 return;
    //                             }
    //                         };
    //                         (x, last_price)
    //                     }
    //                     None => {
    //                         error!("No last traded price found");
    //                         return;
    //                     }
    //                 };
    //                 let last_traded_price: f64 = match value.get("data") {
    //                     Some(data) => match data.get("lastPrice") {
    //                         Some(Value::String(last_price)) => last_price.parse(),
    //                         _ => {
    //                             error!("Can't parse last traded price");
    //                             return;
    //                         }
    //                     },
    //                     None => {
    //                         error!("No last traded price found");
    //                         return;
    //                     }
    //                 }
    //                 .expect("Last traded price must be not null");

    //                 debug!(
    //                     "TICKER: {}; LAST PRICE: {}; USD PRICE: {}",
    //                     ticker, last_traded_price, usd_price
    //                 );
    //             }
    //         }
    //         _ => error!("NO TOPIC: {}", value),
    //     }
}
