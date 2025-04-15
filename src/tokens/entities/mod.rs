use super::{TokenSymbol, types::TokenId};
use alloy::primitives::{Address, Uint};
use std::fmt::Display;

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct Token {
    pub(in crate::tokens) id: TokenId,
    pub(in crate::tokens) token_symbol: TokenSymbol,
    pub(in crate::tokens) address: Address,
    pub(in crate::tokens) decimals: u8,
    pub(in crate::tokens) total_supply: Uint<256, 4>,
}

impl Token {
    pub fn new(
        id: TokenId,
        token_symbol: String,
        address: Address,
        decimals: u8,
        total_supply: Uint<256, 4>,
    ) -> Self {
        Self { id, token_symbol, address, decimals, total_supply }
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
        // TODO: implement
        todo!()
    }
}
