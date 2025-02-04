// //TODO:
// // - refactor and split it to modules
// // - create list particular user actions history function
// use std::{
//     cell::{Cell, RefCell},
//     collections::HashMap,
//     rc::Rc,
// };

// use alloy::hex;
// use bigdecimal::BigDecimal;
// use bot::{
//     accounts::data_store::UserDataStore,
//     app::entities::ERC20TokenEntity,
//     configuration::persistent_config::config_get,
//     persistence::{
//         accounts::{self, AccountCollateralDto, AccountDebtDto, AccountInsertDto},
//         borrow::{get_borrows_sorted_by_block, BorrowEventDto},
//         configuration::{self, ConfigurationKey},
//         liquidation::{get_liquidations_sorted_by_block, LiquidationEventDto},
//         repay::{get_repays_sorted_by_block, RepayEventDto},
//         supply::{get_supply_events, SupplyEventDto},
//         tokens::{self, get_tokens, ERC20TokenDto},
//         withdraw::{get_withdraw_events, WithdrawEventDto},
//         PostgresPool,
//     },
//     setup::{self, Application},
// };
// use eyre::Result;
// use tracing::{debug, error};

// // type ReserveAsset = Vec<u8>;

// // const TEST: bool = true;

// // pub enum Event {
// //     Supply(SupplyEventDto),
// //     Borrow(BorrowEventDto),
// //     Repay(RepayEventDto),
// //     Withdraw(WithdrawEventDto),
// //     Liquidation(LiquidationEventDto),
// // }

// // struct Asset {
// //     reserve: ReserveAsset,
// //     amount: BigDecimal,
// // }

// // impl Asset {
// //     pub fn new(reserve: Vec<u8>) -> Self {
// //         Self {
// //             reserve,
// //             amount: BigDecimal::from(0),
// //         }
// //     }
// // }

// // #[derive(Default)]
// // struct UserData {
// //     collateral: HashMap<ReserveAsset, Asset>,
// //     debt: HashMap<ReserveAsset, Asset>,
// // }

// // impl UserData {
// //     pub fn handle_supply(&mut self, event: &SupplyEventDto) {
// //         let collateral_asset = self
// //             .collateral
// //             .entry(event.reserve.clone())
// //             .or_insert_with(|| Asset::new(event.reserve.clone()));
// //         let on_behalf_of = hex::encode(&event.on_behalf_of);
// //         let user = hex::encode(&event.user);
// //         debug!("\tuser: {}; on behalf of: {}", user, on_behalf_of);
// //         debug!("collateral asset before: {}", collateral_asset.amount);
// //         debug!("event amount: {}", event.amount);
// //         collateral_asset.amount = &collateral_asset.amount + &event.amount;
// //         debug!("collateral asset after: {}", collateral_asset.amount);
// //     }

// //     pub fn handle_borrow(&mut self, event: &BorrowEventDto) {
// //         let asset = self
// //             .debt
// //             .entry(event.reserve.clone())
// //             .or_insert_with(|| Asset::new(event.reserve.clone()));
// //         let on_behalf_of = hex::encode(&event.on_behalf_of);
// //         let user = hex::encode(&event.user);
// //         debug!("\tuser: {}; on behalf of: {}", user, on_behalf_of);
// //         debug!("debt asset before: {}", asset.amount);
// //         debug!("event amount: {}", event.amount);
// //         asset.amount = &asset.amount + &event.amount;
// //         debug!("debt asset after: {}", asset.amount);
// //     }

// //     pub fn handle_withdraw(&mut self, event: &WithdrawEventDto) {
// //         let asset = self
// //             .collateral
// //             .entry(event.reserve.clone())
// //             .or_insert_with(|| Asset::new(event.reserve.clone()));
// //         let to = hex::encode(&event.to);
// //         let user = hex::encode(&event.user);
// //         debug!("\tuser: {}; to: {}", user, to);
// //         debug!("collateral asset before: {}", asset.amount);
// //         debug!("event amount: {}", event.amount);
// //         asset.amount = &asset.amount - &event.amount;
// //         debug!("collateral asset after: {}", asset.amount);
// //     }

// //     pub fn handle_repay(&mut self, event: &RepayEventDto) {
// //         let asset = self
// //             .debt
// //             .entry(event.reserve.clone())
// //             .or_insert_with(|| Asset::new(event.reserve.clone()));

// //         let repayer = hex::encode(&event.repayer);
// //         let user = hex::encode(&event.user);
// //         debug!("\tuser: {}; repayer: {}", user, repayer);
// //         debug!("\tdebt asset before: {}", asset.amount);
// //         debug!("\tevent amount: {}", event.amount);
// //         asset.amount = &asset.amount - &event.amount;
// //         debug!("\tdebt asset after: {}", asset.amount);
// //     }

// //     pub fn handle_liquidation(&mut self, event: &LiquidationEventDto) {
// //         let debt_asset = self
// //             .debt
// //             .entry(event.debt_asset.clone())
// //             .or_insert_with(|| Asset::new(event.debt_asset.clone()));
// //         let collateral_asset = self
// //             .collateral
// //             .entry(event.collateral_asset.clone())
// //             .or_insert_with(|| Asset::new(event.collateral_asset.clone()));

// //         debug!("debt asset before: {}", debt_asset.amount);
// //         debug!("event amount: {}", event.debt_to_cover);
// //         debt_asset.amount = &debt_asset.amount - &event.debt_to_cover;
// //         debug!("debt asset after: {}", debt_asset.amount);
// //         debug!("collateral asset before: {}", collateral_asset.amount);
// //         debug!("event amount: {}", event.liquidated_collateral_amount);
// //         collateral_asset.amount = &collateral_asset.amount - &event.liquidated_collateral_amount;
// //         debug!("collateral asset after: {}", collateral_asset.amount);
// //     }
// // }

// // #[derive(Default)]
// // struct UserDataStore {
// //     pub data: HashMap<Vec<u8>, UserData>,
// // }

// // impl UserDataStore {
// //     pub fn handle_event(&mut self, event: Event) {
// //         let x = hex::decode("523B27014B865C4D8F3DAC9C6E3B1BAD13D22B87").unwrap();
// //         let test_user = hex::encode(x);
// //         match event {
// //             Event::Supply(event) => {
// //                 if TEST
// //                     && (hex::encode(event.user.clone()) != test_user
// //                         && hex::encode(event.on_behalf_of.clone()) != test_user)
// //                 {
// //                     return;
// //                 }
// //                 debug!("Handle supply");
// //                 let on_behalf_of = hex::encode(event.on_behalf_of.clone());
// //                 let user = hex::encode(event.user.clone());
// //                 debug!("\tOn behalf of: {}; user: {}", on_behalf_of, user);
// //                 let user_data = if user != on_behalf_of {
// //                     self.data.entry(event.on_behalf_of.clone()).or_default()
// //                 } else {
// //                     self.data.entry(event.user.clone()).or_default()
// //                 };
// //                 user_data.handle_supply(&event);
// //             }
// //             Event::Borrow(event) => {
// //                 if TEST
// //                     && (hex::encode(event.user.clone()) != test_user
// //                         && hex::encode(event.on_behalf_of.clone()) != test_user)
// //                 {
// //                     return;
// //                 }
// //                 debug!("Handle borrow");

// //                 let on_behalf_of = hex::encode(event.on_behalf_of.clone());
// //                 let user = hex::encode(event.user.clone());
// //                 debug!("\tOn behalf of: {}; user: {}", on_behalf_of, user);
// //                 let user_data = if event.user != event.on_behalf_of {
// //                     self.data.entry(event.on_behalf_of.clone()).or_default()
// //                 } else {
// //                     self.data.entry(event.user.clone()).or_default()
// //                 };
// //                 user_data.handle_borrow(&event);
// //             }
// //             Event::Repay(event) => {
// //                 if TEST && hex::encode(event.user.clone()) != test_user {
// //                     return;
// //                 }
// //                 debug!("Handle repay");
// //                 let user_data = self.data.entry(event.user.clone()).or_default();
// //                 user_data.handle_repay(&event);
// //             }
// //             Event::Withdraw(event) => {
// //                 if TEST && hex::encode(event.user.clone()) != test_user {
// //                     return;
// //                 }
// //                 debug!("Handle withdraw");
// //                 let user_data = self.data.entry(event.user.clone()).or_default();
// //                 user_data.handle_withdraw(&event);
// //             }
// //             Event::Liquidation(event) => {
// //                 if TEST && hex::encode(event.user.clone()) != test_user {
// //                     return;
// //                 }
// //                 debug!("Handle liquidation");
// //                 let user_data = self.data.entry(event.user.clone()).or_default();
// //                 user_data.handle_liquidation(&event);
// //             }
// //         }
// //     }
// // }

// #[tokio::main]
// pub async fn main() -> Result<()> {
//     let Application { rpc_rate_limiter } = setup::configure_app().await?;
//     let last_handled_block: i32 = config_get(ConfigurationKey::LastHandledBlock)
//         .await?
//         .parse()?;
//     let last_user_balances_block: i32 = if !TEST {
//         configuration::get(&db_pool, ConfigurationKey::LastUserBalancesBlock)
//             .await?
//             .parse()?
//     } else {
//         1
//     };

//     let mut user_data_store: UserDataStore = UserDataStore::default();

//     let from_block = last_user_balances_block;
//     let to_block = last_handled_block;
//     let events = get_events(&db_pool, from_block, to_block).await?;
//     for event in events {
//         user_data_store.handle_event(event);
//     }

//     let tokens: HashMap<Vec<u8>, ERC20TokenEntity> = get_tokens(&db_pool)
//         .await?
//         .into_iter()
//         .map(|token| (token.address.clone(), token))
//         .collect();

//     let account_dtos: Vec<AccountInsertDto> = user_data_store
//         .data
//         .iter()
//         .map(|cur| AccountInsertDto {
//             address: cur.0.clone(),
//         })
//         .collect();
//     if !account_dtos.is_empty() {
//         let account_ids_by_address: HashMap<Vec<u8>, i32> =
//             accounts::save_accounts(&db_pool, account_dtos)
//                 .await?
//                 .into_iter()
//                 .map(|cur| (cur.address, cur.id))
//                 .collect();

//         let (collateral_dtos, debt_dtos) =
//             user_data_store
//                 .data
//                 .iter()
//                 .fold((Vec::new(), Vec::new()), |mut acc, cur| {
//                     let account_id = account_ids_by_address
//                         .get(cur.0)
//                         .expect("Account id from db entity not found");
//                     let collateral_dtos = cur.1.collateral.iter().map(|collateral| {
//                         let token = tokens
//                             .get(&collateral.1.reserve)
//                             .expect("token for collateral not found");
//                         AccountCollateralDto {
//                             account_id: *account_id,
//                             asset_id: token.id,
//                             amount: collateral.1.amount.clone(),
//                         }
//                     });
//                     let debt_dtos = cur.1.debt.iter().map(|debt| {
//                         let token = tokens
//                             .get(&debt.1.reserve)
//                             .expect("token for debt not found");
//                         AccountDebtDto {
//                             account_id: *account_id,
//                             asset_id: token.id,
//                             amount: debt.1.amount.clone(),
//                         }
//                     });
//                     acc.0.extend(collateral_dtos);
//                     acc.1.extend(debt_dtos);
//                     acc
//                 });

//         accounts::save_collaterals(&db_pool, collateral_dtos).await?;
//         accounts::save_debts(&db_pool, debt_dtos).await?;
//         configuration::set(
//             &db_pool,
//             ConfigurationKey::LastUserBalancesBlock,
//             &last_handled_block.to_string(),
//         )
//         .await?;
//     }

//     let user_count = user_data_store.data.keys().len();
//     for (user_address, user_data) in user_data_store.data {
//         let collateral_tokens: Vec<(&ERC20TokenEntity, Asset)> = user_data
//             .collateral
//             .into_iter()
//             .map(|asset| {
//                 let token = tokens.get(&asset.1.reserve);
//                 if token.is_none() {
//                     let token_address = hex::encode(asset.0);
//                     error!("Token not found: {}", token_address)
//                 }
//                 (token.unwrap(), asset.1)
//             })
//             .collect();
//         let debt_tokens: Vec<(&ERC20TokenEntity, Asset)> = user_data
//             .debt
//             .into_iter()
//             .map(|asset| {
//                 let token = tokens.get(&asset.1.reserve);
//                 if token.is_none() {
//                     let token_address = hex::encode(asset.0);
//                     error!("Token not found: {}", token_address)
//                 }
//                 (token.unwrap(), asset.1)
//             })
//             .collect();
//         let user_address = hex::encode(user_address);

//         println!("USER: {}", user_address);
//         println!("COLLATERAL:");
//         for (token, asset) in collateral_tokens {
//             let divisor = 10.0_f64.powi(token.decimals);
//             let amount = asset.amount / divisor;
//             println!("\t{}: {}", token.symbol, amount);
//         }
//         println!("DEBT:");
//         for (token, asset) in debt_tokens {
//             let divisor = 10.0_f64.powi(token.decimals);
//             let amount = asset.amount / divisor;
//             println!("\t{}: {}", token.symbol, amount);
//         }
//     }

//     println!("Amount of users: {}", user_count);

//     Ok(())
// }
