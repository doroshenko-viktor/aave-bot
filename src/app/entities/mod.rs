use alloy::primitives::{Address, Uint};

#[derive(Debug, sqlx::FromRow)]
pub struct AccountEntity {
    pub id: i32,
    pub address: Vec<u8>,
}

#[derive(Debug, sqlx::FromRow)]
pub struct ERC20TokenEntity {
    pub id: i32,
    pub address: Address,
    pub name: Option<String>,
    pub symbol: String,
    pub decimals: i32,
    pub total_supply: Uint<256, 4>,
}
