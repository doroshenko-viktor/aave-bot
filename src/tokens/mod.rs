pub mod asset_rules_aggregator;
pub mod blockchain_client;
pub mod entities;
mod persistence;
pub mod token_service;
pub mod types;

use alloy::{primitives::Address, sol};
use bigdecimal::BigDecimal;

pub type TokenSymbol = String;

/*
Convenence alias for address of token - not type safe and not ensures underlying address to be address of the token only;
This is needed only for readability purposes
*/
pub type TokenAddress = Address;

sol! {
    #[sol(rpc)]
    #[derive(Debug)]
    interface IERC20 {
        function name() external view returns (string);
        function symbol() external view returns (string);
        function decimals() external view returns (uint8);
        function totalSupply() external view returns (uint256);
    }
}

#[derive(Debug, sqlx::FromRow)]
pub struct ERC20TokenDto {
    pub address: TokenAddress,
    pub name: Option<String>,
    pub symbol: TokenSymbol,
    pub decimals: i32,
    pub total_supply: BigDecimal,
}
