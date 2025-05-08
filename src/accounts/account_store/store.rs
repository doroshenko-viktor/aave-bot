use crate::{
    accounts::entities::{UserAccount, UserAccountRef},
    tokens::types::TokenId,
    utils::result_utils::CollectionResult,
};
use crate::{
    configuration::globals::get_global_settings,
    shared::persistence::AccountAddress,
};
use dashmap::DashMap;
use futures::{StreamExt, future::join_all};
use std::{
    collections::HashMap,
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
};
use tokio::sync::RwLock;
use tracing::error;

#[derive(Debug, Default)]
pub struct Meta {
    pub last_block_number: AtomicU64,
}

#[derive(Clone)]
pub struct UserAccountStore {
    /* Map of user accounts by their address in blockchain */
    pub(in crate::accounts) accounts: DashMap<AccountAddress, UserAccountRef>,
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

    //TODO: move low hf accounts to external struct, so it may be observed not in the same lock
    //as this struct
    low_hf_accounts: Arc<RwLock<HashMap<TokenId, Vec<AccountAddress>>>>, // TODO: maybe use
    pub(in crate::accounts) meta: Arc<Meta>,
}

impl UserAccountStore {
    pub fn new(
        accounts: DashMap<AccountAddress, UserAccountRef>,
        last_block: u64,
    ) -> Self {
        Self {
            accounts,
            low_hf_accounts: Arc::new(RwLock::new(HashMap::new())),
            meta: Arc::new(Meta {
                last_block_number: AtomicU64::new(last_block),
            }),
        }
    }

    pub fn get_last_block_number(&self) -> u64 {
        self.meta.last_block_number.load(Ordering::SeqCst)
    }

    pub fn set_last_block_number(&self, last_block_number: u64) {
        self.meta
            .last_block_number
            .fetch_max(last_block_number, Ordering::SeqCst);
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
                HashMap::<TokenId, Vec<_>>::new(),
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

    pub async fn get_user_account(
        &self,
        account_address: &AccountAddress,
    ) -> UserAccountRef {
        if let Some(account_ref) = self.accounts.get(account_address) {
            account_ref.clone()
        } else {
            let user_account = Arc::new(RwLock::new(
                UserAccount::new_with_address(account_address.clone()),
            ));
            self.accounts.insert(*account_address, user_account.clone());
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
                Ok(x) => acc.results.push(x),
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
        .results
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
