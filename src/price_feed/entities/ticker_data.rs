use crate::tokens::entities::Token;

#[derive(Debug, Clone)]
pub struct TickerData {
    pub token_1: &'static Token,
    pub token_2: &'static Token,
    pub bybit_repr: String,
    pub chainlink_repr: String,
    pub pyth_repr: String,
}

impl TickerData {
    pub fn new(
        token_1: &'static Token,
        token_2: &'static Token,
        bybit_repr: String,
        chainlink_repr: String,
        pyth_repr: String,
    ) -> Self {
        Self {
            token_1,
            token_2,
            bybit_repr,
            chainlink_repr,
            pyth_repr,
        }
    }
}
