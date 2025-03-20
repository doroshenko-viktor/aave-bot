mod ticker;
mod ticker_data;
mod ticker_price;

use bigdecimal::BigDecimal;

pub use ticker::TickerId;
pub use ticker_data::TickerData;

pub type TickerSymbol = String;
pub type TokenUsdPrice = BigDecimal;
