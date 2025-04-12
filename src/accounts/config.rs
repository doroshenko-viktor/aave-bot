use bigdecimal::BigDecimal;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct AccountsConfig {
    pub low_health_factor_threshold: f32,
}
