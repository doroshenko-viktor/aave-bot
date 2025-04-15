use aave_bot::{
    configuration::globals,
    setup,
    tokens::{
        self, ERC20TokenSaveDbDto,
        blockchain_client::{AaveProtocolDataProvider, ERC20TokenContract},
    },
};
use alloy::primitives::Address;
use eyre::Result;
use futures::{StreamExt, TryStreamExt};
use sqlx::types::BigDecimal;
use std::str::FromStr;
use tracing::{error, info};

#[tokio::main]
pub async fn main() -> Result<()> {
    setup::configure_app().await?;
    let config = globals::get_global_settings();
    let db_pool = globals::get_db_pool();
    let rpc_provider = setup::get_new_rpc_provider(
        &globals::get_global_settings().rpc_config,
    )?;

    let aave_data_provider = AaveProtocolDataProvider::new(
        rpc_provider.clone(),
        Address::from_str(
            &config.blockchain.aave_v3_pool_data_provider_address,
        )?,
    );

    let aave_tokens = aave_data_provider.get_aave_tokens().await?;
    let token_db_dtos = futures::stream::iter(aave_tokens)
        .then(async |token| {
            info!(
                "Processing token: {:?}::{:?}",
                token.symbol, token.tokenAddress
            );

            let reserve_token_data = aave_data_provider
                .get_reserve_token_details(token.tokenAddress)
                .await?;

            let erc_20_contract = ERC20TokenContract::new(
                rpc_provider.clone(),
                token.tokenAddress,
            );
            let erc20_token_dto = erc_20_contract.get_token_details().await?;

            let token_db_dto = ERC20TokenSaveDbDto {
                address: token.tokenAddress.to_vec(),
                name: erc20_token_dto.name,
                symbol: erc20_token_dto.symbol,
                decimals: erc20_token_dto.decimals,
                total_supply: BigDecimal::from_str(
                    &erc20_token_dto.total_supply.to_string(),
                )?,
                a_token_address: reserve_token_data.aTokenAddress.to_vec(),
                stable_debt_token_address: reserve_token_data
                    .stableDebtTokenAddress
                    .to_vec(),
                variable_debt_token_address: reserve_token_data
                    .variableDebtTokenAddress
                    .to_vec(),
            };
            Ok(token_db_dto)
        })
        .try_collect::<Vec<ERC20TokenSaveDbDto>>()
        .await;

    let token_db_dtos = match token_db_dtos {
        Ok(token_db_dtos) => token_db_dtos,
        Err(e) => {
            error!("Error retrieving token data: {}", e);
            return Err(e);
        }
    };

    info!("Saving tokens to db");
    tokens::persistence::save_tokens(db_pool, token_db_dtos).await?;

    Ok(())
}
