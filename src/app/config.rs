use std::ops::Deref;

use serde::Deserialize;

use crate::{accounts::AccountsConfig, price_feed::PriceFeedConfig};

#[derive(Debug, Deserialize, Clone)]
pub struct AppConfig {
    pub indexing_step: u64,
    pub price_feed: PriceFeedConfig,
    pub accounts: AccountsConfig,
}

impl Deref for AppConfig {
    type Target = PriceFeedConfig;

    fn deref(&self) -> &Self::Target {
        &self.price_feed
    }
}
