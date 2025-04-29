use alloy::primitives::Uint;
use eyre::Result;
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
        let (bigint, _) = self.into_bigint_and_exponent();

        if bigint.sign() == Sign::Minus {
            return Err(eyre::eyre!(
                "Negative value cannot be converted to Uint"
            ));
        }

        let bytes = bigint.to_bytes_le().1;
        let mut array = [0u8; 32];
        let len = bytes.len().min(32);
        array[..len].copy_from_slice(&bytes[..len]);

        Ok(Uint::from_le_bytes(array))
    }
}
