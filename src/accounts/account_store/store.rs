use alloy::{primitives::TxHash, providers::Provider};
use dashmap::DashMap;
use eyre::{Context, Result};
use futures::{StreamExt, future::join_all};
use std::{collections::HashMap, fmt::Display, sync::Arc};
use tokio::sync::RwLock;
use tracing::{debug, error, info};

use crate::{
    accounts::entities::{UserAccount, UserAccountRef},
    blockchain::BlockchainDataProviderFacade,
    blockchain_events::{supply::SupplyEventDto, withdraw::WithdrawEventDto},
    tokens::{
        blockchain::erc20_transfer::Erc20TransferEventDto, token_service,
        types::TokenId,
    },
    utils::{result_utils::CollectionResult, uint_ext::UintExt},
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

#[derive(Clone)]
pub struct UserAccountStore<P>
where
    P: BlockchainDataProviderFacade,
{
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
    low_hf_accounts: Arc<RwLock<HashMap<TokenId, Vec<AccountAddress>>>>, // TODO: maybe use
    // &[Option<Vec<AccountAddress>>; TOKENS_COUNT]
    pub(in crate::accounts) meta: Meta,
    blockchain_facade: P,
}

impl<P> UserAccountStore<P>
where
    P: BlockchainDataProviderFacade,
{
    pub fn new(
        accounts: DashMap<AccountAddress, UserAccountRef>,
        last_block: u64,
        blockchain_facade: P,
    ) -> Self {
        Self {
            accounts,
            low_hf_accounts: Arc::new(RwLock::new(HashMap::new())),
            blockchain_facade,
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

    pub async fn handle_event(&self, event: BlockchainEvent) -> Result<()> {
        info!(
            "Handling log: {} for block: {}",
            event.log_number, event.block_number
        );
        debug!("Event: {:#?}", event);
        let transaction_hash = event.transaction_hash;
        let block_number = event.block_number;
        match event.body {
            BlockchainEventBody::Supply(event_body) => {
                self.handle_supply_event(block_number, event_body).await?;
            }
            BlockchainEventBody::Borrow(event_body) => {
                self.handle_borrow_event(event_body).await;
            }
            BlockchainEventBody::Repay(event_body) => {
                self.handle_repay_event(block_number, event_body).await;
            }
            BlockchainEventBody::Withdraw(event_body) => {
                self.handle_withdraw_event(
                    transaction_hash,
                    block_number,
                    event_body,
                )
                .await?;
            }
            BlockchainEventBody::Liquidation(event_body) => {
                self.handle_liquidation_event(block_number, event_body)
                    .await?;
            }
            BlockchainEventBody::Erc20Transfer(event_body) => {
                self.handle_a_token_transfer(block_number, event_body)
                    .await?;
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

        Ok(())
    }

    async fn handle_liquidation_event(
        &self,
        block_number: u64,
        event_body: crate::blockchain_events::liquidation::LiquidationEventDto,
    ) -> Result<(), eyre::Error> {
        info!(
            "Handling LIQUIDATION for user: {:#x}",
            event_body.user
        );
        let user_data = self.get_user_account(&event_body.user).await;
        let debt_token =
            token_service::get_token_by_address(&event_body.debt_asset);
        let collateral_token =
            token_service::get_token_by_address(&event_body.collateral_asset);
        let a_token_liquidated_amount = self
            .blockchain_facade
            .get_a_token_valuation(
                collateral_token.a_token_address,
                event_body.liquidated_collateral_amount,
                block_number,
            )
            .await
            .wrap_err("Failed to get aToken amount for liquidation event")?;
        user_data.write().await.liquidate(
            debt_token.get_id(),
            collateral_token.get_id(),
            event_body.debt_to_cover,
            event_body.liquidated_collateral_amount,
            a_token_liquidated_amount,
        )?;
        Ok(())
    }

    async fn handle_a_token_transfer(
        &self,
        block_number: u64,
        event_body: Erc20TransferEventDto,
    ) -> Result<(), eyre::Error> {
        let token = token_service::get_token_by_a_token_address(
            &event_body.token_address,
        );

        info!(
            "Handling ERC20 TRANSFER of {} {} from user: {:#x} to: {:#x}",
            token.get_repr(event_body.value)?,
            token.get_symbol(),
            event_body.from,
            event_body.to,
        );

        let native_token_amount = self
            .blockchain_facade
            .get_native_token_valuation(
                token.address,
                event_body.value,
                block_number,
            )
            .await
            .wrap_err("Failed to get aToken amount for transfer event")?;

        let from_user_data = self.get_user_account(&event_body.from).await;
        let mut from_user_data = from_user_data.write().await;

        from_user_data.handle_transfer_out(
            token,
            native_token_amount,
            event_body.value,
        )?;

        if event_body.to.address
            != get_global_settings()
                .blockchain
                .aave_v3_paraswap_liquidity_swap_address
        {
            // in case of transfer to paraswap liquidity swap adapter we have a collateral asset
            // swap and we don't need to add transfered amount to aave swap contract address.
            // later there will be an event for this account with supply of swapped token
            let to_user_data = self.get_user_account(&event_body.to).await;
            let mut to_user_data = to_user_data.write().await;
            to_user_data.handle_transfer_in(
                token,
                native_token_amount,
                event_body.value,
            )?;
        }

        Ok(())
    }

    async fn handle_repay_event(
        &self,
        block_number: u64,
        event_body: crate::blockchain_events::repay::RepayEventDto,
    ) {
        info!(
            "Handling REPAY for user: {:#x}",
            event_body.user
        );
        let user_data = self.get_user_account(&event_body.user).await;
        let token = token_service::get_token_by_address(&event_body.reserve);
        if let Err(err) =
            user_data.write().await.repay(token, event_body.amount)
        {
            error!(
                "err: {}; block_number: {}",
                err, block_number
            );
        }
    }

    async fn handle_borrow_event(
        &self,
        event_body: crate::blockchain_events::borrow::BorrowEventDto,
    ) {
        info!(
            "Handling borrow for user: {:#x}; on behalf of: {:#x}",
            event_body.user, event_body.on_behalf_of
        );
        let user_data = self.get_user_account(&event_body.on_behalf_of).await;

        let reserve_token =
            token_service::get_token_by_address(&event_body.reserve);
        user_data
            .write()
            .await
            .borrow(reserve_token, event_body.amount);
    }

    async fn handle_supply_event(
        &self,
        block_number: u64,
        event_body: SupplyEventDto,
    ) -> Result<()> {
        let token = token_service::get_token_by_address(&event_body.reserve); // TODO: move token
        info!(
            "\tHandling SUPPLY of {}{} for user: {:#x} on behalf of: {:#x};",
            token.get_repr(event_body.amount).unwrap(),
            token.get_symbol(),
            event_body.user,
            event_body.on_behalf_of,
        );
        let user_account =
            self.get_user_account(&event_body.on_behalf_of).await;
        {
            let a_token_amount = self
                .blockchain_facade
                .get_a_token_valuation(
                    event_body.reserve,
                    event_body.amount,
                    block_number,
                )
                .await
                .unwrap();

            let mut account_ref = user_account.write().await;

            account_ref.supply(
                token.id,
                event_body.amount,
                a_token_amount,
            );
        }
        Ok(())
    }

    async fn handle_withdraw_event(
        &self,
        transaction_hash: TxHash,
        block_number: u64,
        event_body: WithdrawEventDto,
    ) -> Result<(), eyre::Error> {
        // if it is withdraw for AAVE ParaSwap Liquidity Swap Adapter
        // check transaction and find initiator address
        // subtract amount in body of token in body from this address
        let account_address = if event_body.user.address
            == get_global_settings()
                .blockchain
                .aave_v3_wrapped_token_gateway_address
        {
            // in case of native token withdrawal it will go through separate contract and this
            // contract will be receiver and requester of withdraw event. in this case we need
            // to additionally extract real beneficiary of withdrawal from transaction
            return Ok(());
            // info!(
            //     "Handling WITHDRAW for user: {:#x} to gateway contract: {:#x}",
            //     event_body.user, event_body.to
            // );
            // let transaction = self
            //     .blockchain_facade
            //     .get_transaction(transaction_hash)
            //     .await?;
            // debug!(
            //     "Transaction {:#?} from: {:#x?} to: {:#x?} inner: {:#?}",
            //     transaction_hash,
            //     transaction.from,
            //     transaction.to,
            //     transaction.inner
            // );
            //
            // AccountAddress::new(transaction.from)
        } else if event_body.user.address
            == get_global_settings()
                .blockchain
                .aave_v3_paraswap_liquidity_swap_address
        {
            // in case of withdrawal from/to paraswap liquidity swap adapter we need to ignore it
            // to avoid double counting of amount in user account.
            // this is actually not a withdrawal from user account as before this withdrawal there
            // was a transfer of collateral from user account to paraswap contract, which we
            // already counted in handle_a_token_transfer method
            return Ok(());
        } else {
            // user from event is the real invoker
            event_body.user
        };

        let user_data = self.get_user_account(&account_address).await;
        let token = token_service::get_token_by_address(&event_body.reserve);
        info!(
            "\tHandling WITHDRAW of {}{} for user: {:#x} on behalf of: {:#x};",
            token.get_repr(event_body.amount)?,
            token.get_symbol(),
            event_body.user,
            account_address,
        );
        let a_token_amount = self
            .blockchain_facade
            .get_a_token_valuation(
                token.address,
                event_body.amount,
                block_number,
            )
            .await
            .wrap_err("Failed to get aToken amount for withdrawal event")
            .unwrap();

        let res = user_data.write().await.withdraw(
            token,
            event_body.amount,
            a_token_amount,
        );
        Ok(
            #[cfg(debug_assertions)]
            {
                if let Err(err) = res {
                    error!(
                        "Error happened during handling withdraw event: {}",
                        err
                    );
                    return Err(err);
                }
                ()
            },
        )
    }

    async fn get_user_account(
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
