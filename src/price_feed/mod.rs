mod bybit;
mod chainlink;
mod config;
pub mod entities;
mod persistence;
mod price_feeds_service;
pub mod prices_aggregator;
mod pyth;
mod startup;
pub mod ticker_service;

pub use config::PriceFeedConfig;
use entities::TickerId;
pub use price_feeds_service::PriceFeedsService;
pub use startup::start_price_feed_consumption;
use tokio::sync::mpsc::Sender;

use crate::tokens::entities::Token;

pub type ChangedTokenPriceTX = Sender<&'static Token>;

pub enum PriceFeedSource {
    Bybit,
    Chainlink,
    Pyth,
}
