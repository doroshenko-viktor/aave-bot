use super::ERC20TokenDto;
use crate::utils::uint_ext::UintExt;
use crate::{NodeWsProvider, configuration::RpcRateLimiter, tokens::IERC20};
use eyre::Result;
use std::sync::Arc;
use tracing::debug;

pub async fn get_token_details(
    rpc_provider: &NodeWsProvider,
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
    let total_supply = contract.totalSupply().call().await?.to_big_decimal()?;

    Ok(ERC20TokenDto {
        address,
        name: Some(name),
        symbol,
        decimals: decimals as i32,
        total_supply,
    })
}
