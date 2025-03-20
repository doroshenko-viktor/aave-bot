pub struct PriceFeedConfig {
    pub bybit_spot_url: String,
}

impl PriceFeedConfig {
    pub fn new(bybit_spot_url: String) -> Self {
        Self { bybit_spot_url }
    }
}
