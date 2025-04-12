use super::types::TokenId;

#[derive(Debug, Clone)]
pub struct AssetRules {}

impl AssetRules {
    pub fn get_asset_liquidation_threshold(&self, token_id: TokenId) -> f64 {
        0.8
        // todo: make db config
    }
}
