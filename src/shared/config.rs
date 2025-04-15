use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct BlockchainConfig {
    pub aave_v3_pool_contract_address: String,
    pub aave_v3_pool_data_provider_address: String,
}
