use alloy::primitives::Address;
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct BlockchainConfig {
    pub aave_v3_pool_contract_address: String,
    pub aave_v3_pool_data_provider_address: String,
    pub aave_v3_wrapped_token_gateway_address: Address,
    pub aave_v3_paraswap_liquidity_swap_address: Address,
}
