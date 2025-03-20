use bigdecimal::BigDecimal;

pub type AssetAmount = BigDecimal;

#[derive(Debug, Default)]
pub struct AccountAsset {
    pub amount: AssetAmount,
}
impl AccountAsset {
    pub fn new(amount: AssetAmount) -> Self {
        Self { amount }
    }
}
