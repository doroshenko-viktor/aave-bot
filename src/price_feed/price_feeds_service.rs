use super::{
    bybit::{BybitClientSpot, BybitWsConnectionManager},
    entities::TickerData,
    ticker_service,
};
use eyre::Result;

pub struct PriceFeedsService {
    bybit: BybitWsConnectionManager<BybitClientSpot>,
}

impl PriceFeedsService {
    pub fn new(bybit: BybitWsConnectionManager<BybitClientSpot>) -> Self {
        Self { bybit }
    }

    pub async fn start_feed(&mut self, ticker: TickerData) -> Result<()> {
        //TODO: ?
        self.bybit.subscribe_price(ticker.bybit_repr).await?;

        Ok(())
    }
}

pub async fn start_configured_price_feeds(
    mut price_feeds_service: PriceFeedsService,
) -> Result<()> {
    for ticker in ticker_service::get_active_tickers() {
        let _ = price_feeds_service.start_feed(ticker.1).await;
    }

    Ok(())
}
