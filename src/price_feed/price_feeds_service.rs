use crate::configuration::globals;

use super::{
    bybit::{BybitClientSpot, BybitWsConnectionManager},
    entities::{TickerData, TickerId},
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
