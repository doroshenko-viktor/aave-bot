use std::str::FromStr;

use alloy::primitives::Uint;
use eyre::{Result, eyre};
use num_bigint::{BigInt, Sign};
use sqlx::types::BigDecimal;

pub trait UintExt {
    fn to_decimal(&self) -> BigDecimal;
}

impl UintExt for Uint<256, 4> {
    fn to_decimal(&self) -> BigDecimal {
        let bytes: [u8; 32] = self.to_le_bytes();
        let big_int = BigInt::from_bytes_le(num_bigint::Sign::Plus, &bytes);

        BigDecimal::from(big_int)
    }
}

pub trait DecimalExt {
    fn to_uint(self) -> Result<Uint<256, 4>>;
}

impl DecimalExt for BigDecimal {
    fn to_uint(self) -> Result<Uint<256, 4>> {
        let mut big_decimal_str = self.to_string();

        if big_decimal_str.starts_with('-') {
            return Err(eyre::eyre!(
                "Negative numbers are not supported"
            ));
        }

        if big_decimal_str.contains('e') || big_decimal_str.contains('E') {
            let (mantissa, exponent) = big_decimal_str
                .split_once('e')
                .or_else(|| big_decimal_str.split_once('E'))
                .ok_or_else(|| {
                    eyre::eyre!("Invalid scientific notation format")
                })?;

            let exponent: i64 = exponent
                .parse()
                .map_err(|_| eyre::eyre!("Failed to parse exponent"))?;

            let mantissa = mantissa.replace(".", "");
            let decimal_pos = mantissa
                .chars()
                .position(|c| c == '.')
                .unwrap_or(mantissa.len());
            let digits = mantissa.len() as i64;

            let effective_exponent = exponent + (decimal_pos as i64) - digits;

            if effective_exponent < 0 {
                return Err(eyre::eyre!(
                    "Non-integer BigDecimal not supported"
                ));
            }

            big_decimal_str = format!(
                "{}{}",
                mantissa,
                "0".repeat(effective_exponent as usize)
            );
        }

        Uint::from_str(&big_decimal_str)
            .map_err(|_| eyre::eyre!("Failed to parse to Uint"))
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;
    use sqlx::types::BigDecimal;

    #[test]
    fn test_uint_to_decimal() {
        let uint_value = Uint::<256, 4>::from(123456789);
        let decimal_value = uint_value.to_decimal();
        assert_eq!(decimal_value.to_string(), "123456789");
    }

    #[test]
    fn test_decimal_to_uint_0() {
        let decimal_value = BigDecimal::from_str("1500").unwrap();
        let uint_value = decimal_value.to_uint().unwrap();
        assert_eq!(uint_value, Uint::<256, 4>::from(1500));
    }

    #[test]
    fn test_decimal_to_uint_1() {
        let decimal_value = BigDecimal::from_str("1500e16").unwrap();
        let uint_value = decimal_value.to_uint().unwrap();
        assert_eq!(
            uint_value,
            Uint::<256, 4>::from(15000000000000000000_u128)
        );
    }

    #[test]
    fn test_decimal_to_uint_2() {
        let decimal_value = BigDecimal::from_str("1.5e16").unwrap();
        let uint_value = decimal_value.to_uint().unwrap();
        assert_eq!(
            uint_value,
            Uint::<256, 4>::from(15000000000000000_u128)
        );
    }
}
