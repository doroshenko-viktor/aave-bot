use alloy::primitives::Address;
use dashmap::DashMap;
use eyre::Result;
use futures::{StreamExt, future::join_all};
use std::{collections::HashMap, sync::Arc};
use tokio::sync::RwLock;
use tracing::{debug, error};

use crate::{
    accounts::entities::{UserAccount, UserAccountRef},
    tokens::types::TokenId,
    utils::result_utils::CollectionResult,
};
use crate::{
    blockchain_events::events::{BlockchainEvent, BlockchainEventBody},
    configuration::globals::get_global_settings,
    shared::persistence::AccountAddress,
};

#[derive(Debug, Default, Clone)]
pub struct Meta {
    pub last_block_number: u64,
}

#[derive(Debug, Clone)]
pub struct UserAccountStore {
    /* Map of user accounts by their address in blockchain */
    pub(in crate::accounts) accounts:
        Arc<DashMap<AccountAddress, UserAccountRef>>,
    /* Contains references to account by their assets(collaterals or debts) */
    // TODO: rewrite it to contain only account addresses of low hf accounts and then get them by address and get rid of arc on user account
    // TODO: I also don't need RwLock on hashmap, because I will not change it
    // ✅ 4. Use SlotMap or IndexMap for stable indexing (if needed)
    // If you're dealing with huge account sets and ever need stable, index-based traversal (like for efficient batch processing), using a SlotMap or IndexMap can improve memory locality and iteration performance.
    //
    // If you want all tasks (like your periodic loop or per-token handlers) to react to the latest price without polling, consider:
    // let (tx, rx) = tokio::sync::watch::channel(initial_price);
    // tx.send(new_price) pushes a new price.
    // All rx handles instantly see .changed().await.
    low_hf_accounts: Arc<RwLock<HashMap<TokenId, Vec<AccountAddress>>>>, // TODO: maybe use
    // &[Option<Vec<AccountAddress>>; TOKENS_COUNT]
    pub(in crate::accounts) meta: Meta,
}

impl UserAccountStore {
    pub fn new(
        accounts: Arc<DashMap<AccountAddress, UserAccountRef>>,
        last_block: u64,
    ) -> Self {
        Self {
            accounts,
            low_hf_accounts: Arc::new(RwLock::new(HashMap::new())),
            meta: Meta { last_block_number: last_block },
        }
    }

    pub fn get_last_block_number(&self) -> u64 {
        self.meta.last_block_number
    }

    pub fn set_last_block_number(&mut self, last_block_number: u64) {
        self.meta.last_block_number = last_block_number;
    }

    pub async fn refresh_low_hf_accounts(&self) {
        let low_hf_threshold = &get_global_settings()
            .app
            .accounts
            .low_health_factor_threshold;

        let low_hf_accounts_map =
            futures::stream::iter(self.accounts.iter().map(async |kvp| {
                let health_factor =
                    kvp.value().read().await.calculate_health_factor().await;
                match health_factor {
                    Ok(hf) if &hf < low_hf_threshold => Some(kvp),
                    Err(err) => {
                        error!(
                            "Error during refresh low hf accounts: {}",
                            err
                        );
                        None
                    }
                    _ => None,
                }
            }))
            .buffer_unordered(8)
            .filter_map(async |x| match x {
                Some(val) => {
                    let account = val.value().read().await;
                    Some((
                        account.address,
                        account.get_related_tokens(),
                    ))
                }
                None => None,
            })
            .fold(
                HashMap::<TokenId, Vec<Address>>::new(),
                async |mut result, (account_address, tokens_ids)| {
                    for token_id in tokens_ids {
                        result
                            .entry(token_id)
                            .and_modify(|val| {
                                val.push(account_address);
                            })
                            .or_insert_with(|| vec![account_address.clone()]);
                    }
                    result
                },
            )
            .await;

        // TODO: check performance of these implementations
        // let low_hf_accounts = join_all(self.accounts.iter().map(|kvp| async move {
        //     match calculate_health_factor(kvp.value()).await {
        //         Ok(hf) if &hf < low_hf_threshold => Some(kvp),
        //         Err(err) => {
        //             error!("Error during refresh low hf accounts: {}", err);
        //             None
        //         }
        //         _ => None,
        //     }
        // }))
        // .await;

        // let low_hf_accounts_map = join_all(
        //     low_hf_accounts
        //         .into_iter()
        //         .filter_map(|account| account)
        //         .map(|kvp| async move {
        //             let account = kvp.value();
        //             (account.read().await.get_related_tokens(), account);
        //         }),
        // )
        // .await
        // .into_iter()
        // .fold(
        //     HashMap::<&Token, Vec<UserAccountRef>>::new(),
        //     |mut result, (tokens, account)| {
        //         for token in tokens {
        //             result
        //                 .entry(token)
        //                 .and_modify(|val| {
        //                     val.push(account.clone());
        //                 })
        //                 .or_insert_with(|| vec![account.clone()]);
        //         }
        //         result
        //     },
        // );
        let mut low_hf_accounts_lock = self.low_hf_accounts.write().await;
        *low_hf_accounts_lock = low_hf_accounts_map;
    }

    pub async fn get_accounts_to_liquidate_on_price_change(
        &self,
        token: TokenId,
    ) -> Vec<UserAccountRef> {
        let low_hf_accounts = self.low_hf_accounts.read().await;

        if let Some(account_addresses) = low_hf_accounts.get(&token) {
            let accounts = account_addresses.iter().map(|address| {
                self.accounts
                    .get(address)
                    .expect("Expected to find account in store")
                    .clone()
            });
            get_accounts_to_liquidate(accounts).await
        } else {
            vec![]
        }
    }

    pub async fn handle_event(
        &self,
        event: BlockchainEvent,
    ) -> Result<UserAccountRef> {
        let user_account = match event.body {
            BlockchainEventBody::Supply(event_body) => {
                // debug!(
                //     "\tHandling supply for user: {:#x} on behalf of: {:#x}",
                //     event_body.on_behalf_of, event_body.user
                // );
                let user_account =
                    self.get_user_account(&event_body.on_behalf_of).await;
                {
                    let mut account_ref = user_account.write().await;
                    account_ref.handle_supply(event_body);
                }

                user_account
            }
            BlockchainEventBody::Borrow(event_body) => {
                debug!(
                    "Handling borrow for user: {:#x}; on behalf of: {:#x}",
                    event_body.user, event_body.on_behalf_of
                );
                let user_data =
                    self.get_user_account(&event_body.on_behalf_of).await;
                user_data.write().await.handle_borrow(event_body);
                user_data
            }
            BlockchainEventBody::Repay(event_body) => {
                debug!(
                    "Handling REPAY for user: {:#x}",
                    event_body.user
                );
                let user_data = self.get_user_account(&event_body.user).await;
                user_data.write().await.handle_repay(event_body)?;
                user_data
            }
            BlockchainEventBody::Withdraw(event_body) => {
                debug!(
                    "Handling WITHDRAW for user: {:#x}",
                    event_body.user
                );
                let user_data = self.get_user_account(&event_body.user).await;
                let res = user_data.write().await.handle_withdraw(event_body);
                // TODO: consider separate Error type to take action when handling of event is not successful
                // TODO: e.g. call aave contract to get current status of this account(but think about future events coming for this account - maybe collect fault accounts and request their status after full blockchian scan)
                #[cfg(debug_assertions)]
                {
                    if let Err(err) = res {
                        error!(
                            "Error happened during handling withdraw event: {}",
                            err
                        );
                    }
                }
                user_data
            }
            BlockchainEventBody::Liquidation(event_body) => {
                debug!(
                    "Handling LIQUIDATION for user: {:#x}",
                    event_body.user
                );
                let user_data = self.get_user_account(&event_body.user).await;
                user_data.write().await.handle_liquidation(event_body)?;
                user_data
            }
        };

        //TODO: think about do I need it, considering locking of low hf accounts, while corresponding process will move this account into low hf anyway in couple of seconds and low hf still does not mean it will be liquidated in near future
        // match calculate_health_factor(&user_account, &self.price_aggregator).await {
        //     Ok(account_health_factor) => {
        //         debug!("Health factor : {}", account_health_factor);
        //         let user_account_lock = user_account.read().await;
        //         let account_tokens = user_account_lock.get_related_tokens();

        //         if get_global_settings().accounts.low_health_factor_threshold
        //             > account_health_factor
        //         {
        //             let mut low_hf_accounts_lock = self.low_hf_accounts.write().await;
        //             for token in account_tokens {
        //                 match low_hf_accounts_lock.get_mut(&token) {
        //                     Some(x) => x.pu,
        //                     None => todo!(),
        //                 };
        //             }
        //         }

        //         // TODO: react on low HF
        //     }
        //     Err(e) => todo!("handle error on health factor calc"),
        // }

        Ok(user_account)
    }

    async fn get_user_account(
        &self,
        account_address: &Address,
    ) -> UserAccountRef {
        if let Some(account_ref) = self.accounts.get(account_address) {
            account_ref.clone()
        } else {
            let user_account = Arc::new(RwLock::new(UserAccount::default()));
            self.accounts.insert(
                account_address.clone(),
                user_account.clone(),
            );
            user_account
        }
    }
}

async fn get_accounts_to_liquidate(
    accounts: impl Iterator<Item = UserAccountRef>,
) -> Vec<UserAccountRef> {
    let accounts_with_hf = join_all(accounts.map(|account| async move {
        let health_factor =
            account.read().await.calculate_health_factor().await;
        match health_factor {
            Ok(hf) => Ok((account, hf)),
            Err(e) => Err(e),
        }
    }))
    .await
    .into_iter()
    .fold(
        CollectionResult::<(UserAccountRef, f32)>::new(),
        |mut acc, cur| {
            match cur {
                Ok(x) => acc.result.push(x),
                Err(e) => acc.errors.push(e),
            }
            acc
        },
    );

    for err in accounts_with_hf.errors {
        error!(
            "Error calculating low hf accounts: {}",
            err
        )
    }

    accounts_with_hf
        .result
        .into_iter()
        .filter_map(|(account, hf)| {
            if hf > 1_f32 {
                None
            } else {
                Some(account.clone())
            }
        })
        .collect()
}
