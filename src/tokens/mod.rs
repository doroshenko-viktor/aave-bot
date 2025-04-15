pub mod asset_rules_aggregator;
pub mod blockchain;
pub mod blockchain_client;
pub mod entities;
pub mod persistence;
pub mod token_service;
pub mod types;

use alloy::primitives::{Address, Uint};
use sqlx::types::BigDecimal;

pub type TokenSymbol = String;

/*
Convenence alias for address of token - not type safe and not ensures underlying address to be address of the token only;
This is needed only for readability purposes
*/
pub type TokenAddress = Address;

#[derive(Debug, sqlx::FromRow)]
pub struct ERC20TokenDto {
    pub address: Address,
    pub name: Option<String>,
    pub symbol: TokenSymbol,
    pub decimals: i32,
    pub total_supply: Uint<256, 4>,
}

#[derive(Debug, sqlx::FromRow)]
pub struct ERC20TokenDbDto {
    pub id: i64,
    pub address: Vec<u8>,
    pub name: Option<String>,
    pub symbol: TokenSymbol,
    pub decimals: i32,
    pub total_supply: BigDecimal,
    pub a_token_address: Vec<u8>,
    pub stable_debt_token_address: Vec<u8>,
    pub variable_debt_token_address: Vec<u8>,
}

#[derive(Debug, sqlx::FromRow)]
pub struct ERC20TokenSaveDbDto {
    pub address: Vec<u8>,
    pub name: Option<String>,
    pub symbol: TokenSymbol,
    pub decimals: i32,
    pub total_supply: BigDecimal,
    pub a_token_address: Vec<u8>,
    pub stable_debt_token_address: Vec<u8>,
    pub variable_debt_token_address: Vec<u8>,
}
