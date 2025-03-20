use crate::tokens::entities::Token;

use super::{PriceFeedConfig, bybit};
use super::{entities::TickerId, price_feeds_service::PriceFeedsService};
use eyre::Result;
use tokio::sync::mpsc::Receiver;
use tracing::{error, info, warn};

pub async fn start_price_feed_consumption(
    config: &PriceFeedConfig,
) -> Result<(PriceFeedsService, Receiver<&'static Token>)> {
    let (price_tx, changed_token_price_rx) = tokio::sync::mpsc::channel::<&'static Token>(1000);

    let bybit_connection_manager = bybit::run_bybit_ws_channel::<bybit::BybitClientSpot>(
        config.bybit_spot_url.clone(),
        price_tx,
    )
    .await?;

    Ok((
        PriceFeedsService::new(bybit_connection_manager),
        changed_token_price_rx,
    ))
}
