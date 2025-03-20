use bigdecimal::BigDecimal;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub low_health_factor_threshold: BigDecimal,
}
