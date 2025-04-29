use super::blockchain::contracts::IERC20Fallback::{self, IERC20FallbackInstance};
use super::blockchain::contracts::IERC20::IERC20Instance;
use super::{token_service, ERC20TokenDto};
use crate::shared::blockchain::contracts::aave_protocol_data_provider::AaveProtocolDataProvider::getReserveTokensAddressesReturn;
use crate::shared::blockchain::contracts::aave_protocol_data_provider::IPoolDataProvider::TokenData;
use crate::NodeHttpProvider;
use crate::configuration::globals;
use crate::shared::blockchain;
use crate::shared::blockchain::contracts::aave_protocol_data_provider;
use crate::shared::blockchain::errors::GetEventLogsError;
use crate::tokens::blockchain::contracts::IERC20;
use alloy::eips::BlockNumberOrTag;
use alloy::primitives::Address;
use alloy::rpc::types::{Filter, Log};
use alloy::sol_types::SolEvent;
use eyre::Result;
use futures::{FutureExt, StreamExt, TryStreamExt};
use std::num::NonZero;
use tracing::{debug, error, info};

const GET_TOKEN_DETAILS_CREDITS: u32 = 200;

pub struct AaveProtocolDataProvider {
    contract: aave_protocol_data_provider::AaveProtocolDataProvider::AaveProtocolDataProviderInstance<
        NodeHttpProvider,
    >,
}

impl AaveProtocolDataProvider {
    pub fn new(
        rpc_provider: NodeHttpProvider,
        contract_address: Address,
    ) -> Self {
        let contract =
            aave_protocol_data_provider::AaveProtocolDataProvider::AaveProtocolDataProviderInstance::new(
                contract_address,
                rpc_provider,
            );

        Self { contract }
    }

    pub async fn get_aave_tokens(&self) -> Result<Vec<TokenData>> {
        globals::RPC_RATE_LIMITER
            .until_n_ready(NonZero::new(GET_TOKEN_DETAILS_CREDITS).unwrap())
            .await?;

        let result = self.contract.getAllReservesTokens().call().await?;

        Ok(result)
    }

    pub async fn get_reserve_token_details(
        &self,
        token_address: Address,
    ) -> Result<getReserveTokensAddressesReturn> {
        globals::RPC_RATE_LIMITER
            .until_n_ready(NonZero::new(GET_TOKEN_DETAILS_CREDITS).unwrap())
            .await?;

        debug!(
            "Requesting reserve token details for token address: {:?}",
            token_address
        );
        let res = self
            .contract
            .getReserveTokensAddresses(token_address)
            .call()
            .await?;

        Ok(res)
    }
}

pub struct ERC20TokenContract {
    contract: IERC20Instance<NodeHttpProvider>,
    fallback_contract: IERC20FallbackInstance<NodeHttpProvider>,
}

impl ERC20TokenContract {
    pub fn new(
        rpc_provider: NodeHttpProvider,
        contract_address: Address,
    ) -> Self {
        let contract = IERC20::new(contract_address, rpc_provider.clone());
        let fallback_contract =
            IERC20Fallback::new(contract_address, rpc_provider);
        Self { contract, fallback_contract }
    }

    pub async fn get_token_details(&self) -> Result<ERC20TokenDto> {
        debug!(
            "Requesting token details for address: {:?}",
            self.contract.address()
        );

        globals::RPC_RATE_LIMITER
            .until_n_ready(NonZero::new(GET_TOKEN_DETAILS_CREDITS).unwrap())
            .await?;
        // standart ERC20 contract returns name as string, but some tokens use non standart binary
        // represenation, so need to use fallback contract
        let name = match self.contract.name().call().await {
            Ok(name) => name,
            Err(_) => String::from_utf8(
                self.fallback_contract.name().call().await?.to_vec(),
            )?
            .trim_end_matches('\0')
            .to_string(),
        };
        debug!("Token name: {:?}", name);

        globals::RPC_RATE_LIMITER
            .until_n_ready(NonZero::new(GET_TOKEN_DETAILS_CREDITS).unwrap())
            .await?;
        let symbol = match self.contract.symbol().call().await {
            Ok(symbol) => symbol,
            Err(_) => String::from_utf8(
                self.fallback_contract.symbol().call().await?.to_vec(),
            )?
            .trim_end_matches('\0')
            .to_string(),
        };
        debug!("Token symbol: {:?}", symbol);

        globals::RPC_RATE_LIMITER
            .until_n_ready(NonZero::new(GET_TOKEN_DETAILS_CREDITS).unwrap())
            .await?;
        let decimals = self.contract.decimals().call().await?;
        debug!("Token decimals: {:?}", decimals);

        globals::RPC_RATE_LIMITER
            .until_n_ready(NonZero::new(GET_TOKEN_DETAILS_CREDITS).unwrap())
            .await?;
        let total_supply = self.contract.totalSupply().call().await?;
        debug!("Token total supply: {:?}", total_supply);

        Ok(ERC20TokenDto {
            address: *self.contract.address(),
            name: Some(name),
            symbol,
            decimals: decimals as i32,
            total_supply,
        })
    }
}

pub async fn get_all_tokens_transfer_logs(
    provider: &NodeHttpProvider,
    from_block: u64,
    to_block: u64,
) -> Result<impl Iterator<Item = Log>, GetEventLogsError> {
    let tokens = token_service::get_tokens();
    let res = futures::stream::iter(tokens)
        .then(async |token| {
            get_a_token_transfers(
                provider,
                from_block,
                to_block,
                token.a_token_address,
            )
            .await
        })
        .try_collect::<Vec<_>>()
        .await?
        .into_iter()
        .flatten();

    Ok(res)
}

pub async fn get_a_token_transfers(
    provider: &NodeHttpProvider,
    from_block: u64,
    to_block: u64,
    a_token_address: Address,
) -> Result<Vec<Log>, GetEventLogsError> {
    let filter = Filter::new()
        .address(a_token_address)
        .event_signature(vec![IERC20::Transfer::SIGNATURE_HASH])
        .from_block(BlockNumberOrTag::Number(from_block))
        .to_block(BlockNumberOrTag::Number(to_block));

    debug!(
        "For token address: {} requesting portion of a_token transfer logs between blocks: {} - {}",
        a_token_address, from_block, to_block
    );

    blockchain::logs::get_logs(provider, &filter).await
}
