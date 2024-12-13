use std::{collections::HashSet, str::FromStr, sync::Arc};

use alloy::{primitives::Address, sol};
use bot::{
    persistence::{
        assets::{save_tokens, ERC20TokenDto},
        borrow::get_borrows_sorted_by_block,
        configuration::{self, ConfigurationKey},
        supply::get_supply_events,
    },
    setup::{self, Application},
};

use eyre::{Report, Result};
use futures::future::join_all;
use sqlx::types::BigDecimal;
use tracing::{debug, error};

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

#[tokio::main]
pub async fn main() -> Result<()> {
    let Application {
        db_pool,
        rpc_provider,
        settings,
        rpc_rate_limiter,
    } = setup::configure_app().await?;
    let last_handled_block: i32 = configuration::get(&db_pool, ConfigurationKey::LastHandledBlock)
        .await?
        .parse()?;
    let supply_events = get_supply_events(&db_pool, 0, last_handled_block).await?;
    let borrow_events = get_borrows_sorted_by_block(&db_pool, 0, last_handled_block).await?;
    let addresses: HashSet<Address> = supply_events
        .iter()
        .map(|event| Address::from_slice(&event.reserve))
        .chain(borrow_events.iter().map(|event|Address::from_slice(&event.reserve)))
        .collect();

    let rpc_provider = Arc::new(rpc_provider);
    let rpc_rate_limiter = Arc::new(rpc_rate_limiter);
    let futures = addresses.into_iter().map(|address| {
        let rpc_provider = rpc_provider.clone();
        let rpc_rate_limiter = rpc_rate_limiter.clone();
        async move {
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
            Ok((address, name, symbol, decimals, total_supply))
        }
    });
    let (tokens, errors) =
        join_all(futures)
            .await
            .into_iter()
            .fold((Vec::new(), Vec::new()), |mut acc, cur| {
                match cur {
                    Ok(val) => match BigDecimal::from_str(&val.4._0.to_string())
                        .map_err(|err| Report::new(err))
                    {
                        Ok(total_supply) => {
                            acc.0.push(ERC20TokenDto {
                                address: val.0.to_vec(),
                                name: Some(val.1._0),
                                symbol: val.2._0,
                                decimals: val.3._0 as i32,
                                total_supply,
                            });
                        }
                        Err(err) => acc.1.push(err),
                    },
                    Err(err) => acc.1.push(err),
                }
                acc
            });

    for error in errors {
        error!("{}", error);
    }

    save_tokens(&db_pool, tokens).await?;

    Ok(())
}
