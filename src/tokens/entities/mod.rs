use crate::accounts::types::AssetAmount;

use super::{TokenSymbol, types::TokenId};
use alloy::primitives::{Address, Uint};
use eyre::Result;
use std::{fmt::Display, str::FromStr};

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct Token {
    pub id: TokenId,
    pub token_symbol: TokenSymbol,
    pub address: Address,
    pub decimals: u8,
    pub total_supply: Uint<256, 4>,
    pub a_token_address: Address,
}

impl Token {
    pub fn new(
        id: TokenId,
        token_symbol: String,
        address: Address,
        decimals: u8,
        total_supply: Uint<256, 4>,
        a_token_address: Address,
    ) -> Self {
        Self {
            id,
            token_symbol,
            address,
            decimals,
            total_supply,
            a_token_address,
        }
    }

    pub fn get_repr(&self, value: AssetAmount) -> Result<String> {
        let x = Uint::<256, 4>::from(10u64.pow(self.decimals.into()));
        let integer_part = value / x; // Integer part
        let fractional_part = value % x; // Remainder
        if self.decimals == 0 {
            Ok(integer_part.to_string())
        } else {
            // Format fractional part with leading zeros to match decimals
            let frac_str = format!(
                "{:0width$}",
                fractional_part,
                width = self.decimals as usize
            );
            Ok(format!("{}.{}", integer_part, frac_str))
        }
    }

    pub fn get_symbol(&self) -> String {
        self.token_symbol.clone()
    }

    pub fn get_id(&self) -> TokenId {
        self.id
    }

    pub fn get_decimals(&self) -> u8 {
        self.decimals
    }
}

impl Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Token {{ id: {}, token_symbol: {}, address: {}}}",
            self.id, self.token_symbol, self.address,
        )
    }
}
