use serde::Deserialize;

use crate::{accounts::AccountsConfig, price_feed::PriceFeedConfig};

#[derive(Debug, Deserialize, Clone)]
pub struct AppConfig {
    pub indexing_step: u64,
    pub price_feed: PriceFeedConfig,
    pub accounts: AccountsConfig,
}

impl From<&AppConfig> for PriceFeedConfig {
    fn from(value: &AppConfig) -> Self {
        Self {
            bybit_spot_url: value.price_feed.bybit_spot_url.clone(),
        }
    }
}
