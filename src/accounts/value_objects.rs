use super::types::AssetAmount;

#[derive(Debug, Default)]
pub struct AccountAsset {
    pub amount: AssetAmount,
}

impl AccountAsset {
    pub fn new(amount: AssetAmount) -> Self {
        Self { amount }
    }
}
