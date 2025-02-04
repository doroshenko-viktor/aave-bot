// use std::{collections::HashSet, str::FromStr, sync::Arc};

// use alloy::{primitives::Address, sol};
// use bot::{
//     setup::{self, Application},
//     tokens::blockchain_client::get_token_details,
// };

// use eyre::{Report, Result};
// use futures::future::join_all;
// use sqlx::types::BigDecimal;
// use tracing::{debug, error};

// #[tokio::main]
// pub async fn main() -> Result<()> {
//     let Application { rpc_rate_limiter } = setup::configure_app().await?;
//     let last_handled_block: i32 = configuration::get(&db_pool, ConfigurationKey::LastHandledBlock)
//         .await?
//         .parse()?;
//     let supply_events = get_supply_events(&db_pool, 0, last_handled_block).await?;
//     let borrow_events = get_borrows_sorted_by_block(&db_pool, 0, last_handled_block).await?;
//     let addresses: HashSet<Address> = supply_events
//         .iter()
//         .map(|event| Address::from_slice(&event.reserve))
//         .chain(
//             borrow_events
//                 .iter()
//                 .map(|event| Address::from_slice(&event.reserve)),
//         )
//         .collect();

//     let rpc_provider = Arc::new(rpc_provider);
//     let rpc_rate_limiter = Arc::new(rpc_rate_limiter);
//     let futures = addresses.into_iter().map(|address| {
//         let rpc_provider = rpc_provider.clone();
//         let rpc_rate_limiter = rpc_rate_limiter.clone();
//         async move { get_token_details(&rpc_provider, address, rpc_rate_limiter).await }
//     });
//     let (tokens, errors) =
//         join_all(futures)
//             .await
//             .into_iter()
//             .fold((Vec::new(), Vec::new()), |mut acc, cur| {
//                 match cur {
//                     Ok(val) => acc.0.push(val),
//                     Err(err) => acc.1.push(err),
//                 }
//                 acc
//             });

//     for error in errors {
//         error!("{}", error);
//     }

//     save_tokens(&db_pool, tokens).await?;

//     Ok(())
// }
