use alloy::primitives::Address;
use bigdecimal::BigDecimal;
use dashmap::DashMap;
use eyre::Result;
use futures::{StreamExt, future::join_all};
use std::{collections::HashMap, sync::Arc};
use tokio::sync::RwLock;
use tracing::{debug, error};

use crate::{
    accounts::{
        entities::{UserAccount, UserAccountRef},
        health_factor::calculate_health_factor,
    },
    tokens::entities::Token,
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
    pub(in crate::accounts) accounts: Arc<DashMap<AccountAddress, UserAccountRef>>,
    /* Contains references to account by their assets(collaterals or debts) */
    low_hf_accounts: Arc<RwLock<HashMap<&'static Token, Vec<UserAccountRef>>>>,
    pub(in crate::accounts) meta: Meta,
}

impl UserAccountStore {
    pub fn new(accounts: Arc<DashMap<AccountAddress, UserAccountRef>>, last_block: u64) -> Self {
        Self {
            accounts,
            low_hf_accounts: Arc::new(RwLock::new(HashMap::new())),
            meta: Meta {
                last_block_number: last_block,
            },
        }
    }

    pub fn get_last_block_number(&self) -> u64 {
        self.meta.last_block_number
    }

    pub fn set_last_block_number(&mut self, last_block_number: u64) {
        self.meta.last_block_number = last_block_number;
    }

    pub async fn refresh_low_hf_accounts(&self) {
        let low_hf_threshold = &get_global_settings().accounts.low_health_factor_threshold;

        let low_hf_accounts_map = futures::stream::iter(self.accounts.iter().map(async |kvp| {
            let x = match calculate_health_factor(kvp.value()).await {
                Ok(hf) if &hf < low_hf_threshold => Some(kvp),
                Err(err) => {
                    error!("Error during refresh low hf accounts: {}", err);
                    None
                }
                _ => None,
            };
            x
        }))
        .buffer_unordered(8)
        .filter_map(async |x| match x {
            Some(val) => {
                let account = val.value();
                Some((account.clone(), account.read().await.get_related_tokens()))
            }
            None => None,
        })
        .fold(
            HashMap::<&Token, Vec<UserAccountRef>>::new(),
            async |mut result, (account, tokens)| {
                for token in tokens {
                    result
                        .entry(token)
                        .and_modify(|val| {
                            val.push(account.clone());
                        })
                        .or_insert_with(|| vec![account.clone()]);
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
        token: &'static Token,
    ) -> Vec<UserAccountRef> {
        let low_hf_accounts = self.low_hf_accounts.read().await;

        if let Some(accounts) = low_hf_accounts.get(&token) {
            get_accounts_to_liquidate(accounts).await
        } else {
            vec![]
        }
    }

    pub async fn handle_event(&self, event: BlockchainEvent) -> Result<UserAccountRef> {
        let user_account = match event.body {
            BlockchainEventBody::Supply(event_body) => {
                debug!(
                    "\tHandling supply for user: {:#x} on behalf of: {:#x}",
                    event_body.on_behalf_of, event_body.user
                );
                let user_data = self.get_user_account(&event_body.on_behalf_of).await;
                user_data.write().await.handle_supply(event_body);
                user_data
            }
            BlockchainEventBody::Borrow(event_body) => {
                debug!(
                    "Handling borrow for user: {:#x}; on behalf of: {:#x}",
                    event_body.user, event_body.on_behalf_of
                );
                let user_data = self.get_user_account(&event_body.on_behalf_of).await;
                user_data.write().await.handle_borrow(event_body);
                user_data
            }
            BlockchainEventBody::Repay(event_body) => {
                debug!("Handling REPAY for user: {:#x}", event_body.user);
                let user_data = self.get_user_account(&event_body.user).await;
                user_data.write().await.handle_repay(event_body)?;
                user_data
            }
            BlockchainEventBody::Withdraw(event_body) => {
                debug!("Handling WITHDRAW for user: {:#x}", event_body.user);
                let user_data = self.get_user_account(&event_body.user).await;
                let res = user_data.write().await.handle_withdraw(event_body);
                // TODO: consider separate Error type to take action when handling of event is not successful
                // TODO: e.g. call aave contract to get current status of this account(but think about future events coming for this account - maybe collect fault accounts and request their status after full blockchian scan)
                #[cfg(debug_assertions)]
                {
                    if let Err(err) = res {
                        error!("Error happened during handling withdraw event: {}", err);
                    }
                }
                user_data
            }
            BlockchainEventBody::Liquidation(event_body) => {
                debug!("Handling LIQUIDATION for user: {:#x}", event_body.user);
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

    async fn get_user_account(&self, account_address: &Address) -> UserAccountRef {
        if let Some(user_data) = self.accounts.get(account_address) {
            user_data.clone()
        } else {
            let user_account = Arc::new(RwLock::new(UserAccount::default()));
            self.accounts
                .insert(account_address.clone(), user_account.clone());
            user_account
        }
    }
}

async fn get_accounts_to_liquidate(accounts: &Vec<UserAccountRef>) -> Vec<UserAccountRef> {
    let (accounts_with_hf, errors) = join_all(accounts.iter().map(|account| async move {
        match calculate_health_factor(&account).await {
            Ok(hf) => Ok((account, hf)),
            Err(e) => Err(e),
        }
    }))
    .await
    .into_iter()
    .fold(
        (Vec::with_capacity(accounts.len()), Vec::new()),
        |mut acc, cur| {
            match cur {
                Ok(x) => acc.0.push(x),
                Err(e) => acc.1.push(e),
            }
            acc
        },
    );
    for err in errors {
        error!("{}", err)
    }
    accounts_with_hf
        .into_iter()
        .filter_map(|(account, hf)| {
            if hf > BigDecimal::from(1_i32) {
                None
            } else {
                Some(account.clone())
            }
        })
        .collect()
}
