use alloy::primitives::Uint;
use bigdecimal::BigDecimal;
use std::str::FromStr;

pub trait UintExt {
    fn to_big_decimal(&self) -> eyre::Result<BigDecimal>;
}

impl UintExt for Uint<256, 4> {
    fn to_big_decimal(&self) -> eyre::Result<BigDecimal> {
        Ok(BigDecimal::from_str(&self.to_string())?)
    }
}

pub trait BigDecimalExt {
    fn to_uint(&self) -> eyre::Result<Uint<256, 4>>;
}

impl BigDecimalExt for BigDecimal {
    fn to_uint(&self) -> eyre::Result<Uint<256, 4>> {
        let str_value = self.to_string();
        let value = Uint::<256, 4>::from_str(&str_value)?;
        Ok(value)
    }
}
