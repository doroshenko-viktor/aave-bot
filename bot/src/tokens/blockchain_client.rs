use super::ERC20TokenDto;
use crate::{configuration::RpcRateLimiter, tokens::IERC20, PubSubNodeProvider};
use bigdecimal::BigDecimal;
use eyre::Result;
use std::{str::FromStr, sync::Arc};
use tracing::debug;

pub async fn get_token_details(
    rpc_provider: &PubSubNodeProvider,
    address: alloy::primitives::Address,
    rpc_rate_limiter: Arc<RpcRateLimiter>,
) -> Result<ERC20TokenDto> {
    debug!("{:#?}", address);
    let contract = IERC20::new(address, &rpc_provider);

    rpc_rate_limiter.until_ready().await;
    let name = contract.name().call().await?;
    rpc_rate_limiter.until_ready().await;
    let symbol = contract.symbol().call().await?;
    rpc_rate_limiter.until_ready().await;
    let decimals = contract.decimals().call().await?;
    rpc_rate_limiter.until_ready().await;
    let total_supply = contract.totalSupply().call().await?;

    let total_supply = BigDecimal::from_str(&total_supply._0.to_string())?;

    Ok(ERC20TokenDto {
        address: address.to_vec(),
        name: Some(name._0),
        symbol: symbol._0,
        decimals: decimals._0 as i32,
        total_supply,
    })
}
