pub mod blockchain_client;
pub mod get;
pub mod save;
use alloy::sol;
use bigdecimal::BigDecimal;

use crate::shared::persistence::Address;

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
    pub address: Address,
    pub name: Option<String>,
    pub symbol: String,
    pub decimals: i32,
    pub total_supply: BigDecimal,
}
