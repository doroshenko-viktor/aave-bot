use super::entities::Token;
use bigdecimal::BigDecimal;

#[derive(Debug, Clone)]
pub struct AssetRules {}

impl AssetRules {
    pub async fn get_asset_liquidation_threshold(&self, token: &Token) -> BigDecimal {
        BigDecimal::try_from(0.8).unwrap()
        // todo: make db config
    }
}
