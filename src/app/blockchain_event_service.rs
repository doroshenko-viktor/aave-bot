use alloy::primitives::TxHash;
use eyre::{Context, Result};
use std::sync::Arc;
use tracing::{debug, error, info};

use crate::{
    accounts::UserAccountStore,
    blockchain::BlockchainDataProviderFacade,
    blockchain_events::{
        events::{BlockchainEvent, BlockchainEventBody},
        repay::RepayEventDto,
        supply::SupplyEventDto,
        withdraw::WithdrawEventDto,
    },
    configuration::globals::get_global_settings,
    operational::aave_events::{self, AaveAccountMovement, AaveEventType},
    shared::persistence::AccountAddress,
    tokens::{
        blockchain::erc20_transfer::Erc20TransferEventDto, token_service,
    },
};

pub trait BlockchainEventService {
    fn handle_event(
        &self,
        event: BlockchainEvent,
    ) -> impl Future<Output = Result<Vec<AaveAccountMovement>>>;
}

pub struct BlockchainEventsAccountBalancesService<P>
where
    P: BlockchainDataProviderFacade,
{
    user_account_store: Arc<UserAccountStore>,
    blockchain_facade: P,
}

impl<P> BlockchainEventService for BlockchainEventsAccountBalancesService<P>
where
    P: BlockchainDataProviderFacade,
{
    async fn handle_event(
        &self,
        event: BlockchainEvent,
    ) -> Result<Vec<AaveAccountMovement>> {
        info!(
            "Handling transaction: {:#x}; log: {} for block: {}",
            event.transaction_hash, event.log_number, event.block_number
        );
        debug!("Event: {:#?}", event);
        let transaction_hash = event.transaction_hash;
        let block_number = event.block_number;
        let aave_event = match event.body {
            BlockchainEventBody::Supply(event_body) => {
                self.handle_supply_event(
                    block_number,
                    transaction_hash,
                    event_body,
                )
                .await?
            }
            BlockchainEventBody::Borrow(event_body) => {
                self.handle_borrow_event(
                    block_number,
                    transaction_hash,
                    event_body,
                )
                .await?
            }
            BlockchainEventBody::Repay(event_body) => {
                self.handle_repay_event(
                    block_number,
                    transaction_hash,
                    event_body,
                )
                .await?
            }
            BlockchainEventBody::Withdraw(event_body) => {
                self.handle_withdraw_event(
                    transaction_hash,
                    block_number,
                    event_body,
                )
                .await?
            }
            BlockchainEventBody::Liquidation(event_body) => {
                self.handle_liquidation_event(
                    block_number,
                    transaction_hash,
                    event_body,
                )
                .await?
            }
            BlockchainEventBody::Erc20Transfer(event_body) => {
                self.handle_a_token_transfer(
                    block_number,
                    transaction_hash,
                    event_body,
                )
                .await?
            }
        };
        self.user_account_store.set_last_block_number(block_number);

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

        Ok(aave_event)
    }
}

impl<P> BlockchainEventsAccountBalancesService<P>
where
    P: BlockchainDataProviderFacade,
{
    pub fn new(
        user_account_store: Arc<UserAccountStore>,
        blockchain_facade: P,
    ) -> Self {
        Self { user_account_store, blockchain_facade }
    }

    async fn handle_supply_event(
        &self,
        block_number: u64,
        transaction_hash: TxHash,
        event_body: SupplyEventDto,
    ) -> Result<Vec<AaveAccountMovement>> {
        let token = token_service::get_token_by_address(&event_body.reserve); // TODO: move token
        info!(
            "\tHandling SUPPLY of {}{} for user: {:#x} on behalf of: {:#x};",
            token.get_repr(event_body.amount).unwrap(),
            token.get_symbol(),
            event_body.user,
            event_body.on_behalf_of,
        );
        let a_token_amount = self
            .blockchain_facade
            .get_reserve_token_valuation(
                event_body.reserve,
                event_body.amount,
                block_number,
            )
            .await
            .unwrap();

        let user_account = self
            .user_account_store
            .get_user_account(&event_body.on_behalf_of)
            .await;
        let mut account_ref = user_account.write().await;
        let event_data = AaveAccountMovement::new(
            block_number,
            transaction_hash,
            AaveEventType::Supply,
            account_ref.address,
            token,
            event_body.amount,
            a_token_amount,
        );

        account_ref.supply(token, &event_data);

        Ok(vec![event_data])
    }

    async fn handle_withdraw_event(
        &self,
        transaction_hash: TxHash,
        block_number: u64,
        event_body: WithdrawEventDto,
    ) -> Result<Vec<AaveAccountMovement>, eyre::Error> {
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
            return Ok(vec![]);
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
            return Ok(vec![]);
        } else if event_body.user.address
            == get_global_settings()
                .blockchain
                .aave_v3_paraswap_repay_adapter_address
        {
            // In case of withdraw from paraswap repay adapter it will withdraw some collateral to swap
            // it and repay some debt. We need to find real user address
            let transaction = self
                .blockchain_facade
                .get_transaction(transaction_hash)
                .await?;

            AccountAddress::new(transaction.from)
        } else {
            // user from event is the real invoker
            event_body.user
        };

        let token = token_service::get_token_by_address(&event_body.reserve);
        let a_token_amount = self
            .blockchain_facade
            .get_reserve_token_valuation(
                token.address,
                event_body.amount,
                block_number,
            )
            .await
            .wrap_err("Failed to get aToken amount for withdrawal event")
            .unwrap();

        info!(
            "\tHandling WITHDRAW of {}{};(reserve token: {}) for user: {:#x} on behalf of: {:#x};",
            token.get_repr(event_body.amount)?,
            token.get_symbol(),
            token.get_repr(a_token_amount)?,
            event_body.user,
            account_address,
        );

        let event_body = AaveAccountMovement::new(
            block_number,
            transaction_hash,
            AaveEventType::Withdraw,
            account_address,
            token,
            event_body.amount,
            a_token_amount,
        );
        let account = self
            .user_account_store
            .get_user_account(&account_address)
            .await;
        account.write().await.withdraw(token, &event_body)?;

        Ok(vec![event_body])
    }

    async fn handle_borrow_event(
        &self,
        block_number: u64,
        transaction_hash: TxHash,
        event_body: crate::blockchain_events::borrow::BorrowEventDto,
    ) -> Result<Vec<AaveAccountMovement>> {
        let token = token_service::get_token_by_address(&event_body.reserve);

        info!(
            "Handling borrow of {} {} for user: {:#x}; on behalf of: {:#x}",
            token.get_repr(event_body.amount).unwrap(),
            token.get_symbol(),
            event_body.user,
            event_body.on_behalf_of
        );

        let debt_token_amount = self
            .blockchain_facade
            .get_scaled_borrow_balance(
                event_body.reserve,
                event_body.amount,
                block_number,
            )
            .await?;

        let account_address = event_body.on_behalf_of;
        let event_body = AaveAccountMovement::new(
            block_number,
            transaction_hash,
            AaveEventType::Borrow,
            account_address,
            token,
            event_body.amount,
            debt_token_amount,
        );
        let user_data = self
            .user_account_store
            .get_user_account(&account_address)
            .await;
        user_data.write().await.borrow(token, &event_body)?;

        Ok(vec![event_body])
    }

    async fn handle_repay_event(
        &self,
        block_number: u64,
        transaction_hash: TxHash,
        event_body: RepayEventDto,
    ) -> Result<Vec<AaveAccountMovement>> {
        let token = token_service::get_token_by_address(&event_body.reserve);
        info!(
            "Handling REPAY of {} {} for user: {:#x}",
            token.get_repr(event_body.amount).unwrap(),
            token.get_symbol(),
            event_body.user
        );
        let debt_token_amount = self
            .blockchain_facade
            .get_scaled_borrow_balance(
                event_body.reserve,
                event_body.amount,
                block_number,
            )
            .await?;
        info!(
            "Debt token amount: {}, amount: {}",
            debt_token_amount, event_body.amount
        );

        let account_address = if event_body.user.address
            == get_global_settings()
                .blockchain
                .aave_v3_paraswap_repay_adapter_address
        {
            // In case of repay from paraswap adapter we need to find real user address
            // We also will receive ERC20 transfer event for original user address but it will be
            // ignored as it's transfer to burn address without context of operation
            let transaction = self
                .blockchain_facade
                .get_transaction(transaction_hash)
                .await?;
            AccountAddress::new(transaction.from)
        } else {
            event_body.user
        };
        let event = AaveAccountMovement::new(
            block_number,
            transaction_hash,
            AaveEventType::Repay,
            account_address,
            token,
            event_body.amount,
            debt_token_amount,
        );
        let user_data = self
            .user_account_store
            .get_user_account(&account_address)
            .await;
        user_data.write().await.repay(token, &event)?;

        Ok(vec![event])
    }

    async fn handle_liquidation_event(
        &self,
        block_number: u64,
        transaction_hash: TxHash,
        event_body: crate::blockchain_events::liquidation::LiquidationEventDto,
    ) -> Result<Vec<AaveAccountMovement>, eyre::Error> {
        info!(
            "Handling LIQUIDATION for user: {:#x}",
            event_body.user
        );
        let debt_token =
            token_service::get_token_by_address(&event_body.debt_asset);
        let collateral_token =
            token_service::get_token_by_address(&event_body.collateral_asset);
        let a_token_liquidated_amount = self
            .blockchain_facade
            .get_reserve_token_valuation(
                collateral_token.a_token_address,
                event_body.liquidated_collateral_amount,
                block_number,
            )
            .await
            .wrap_err("Failed to get aToken amount for liquidation event")?;
        let debt_token_amount = self
            .blockchain_facade
            .get_reserve_token_valuation(
                debt_token.address,
                event_body.debt_to_cover,
                block_number,
            )
            .await
            .wrap_err("Failed to get aToken amount for liquidation event")?;

        // TODO: maybe move whole this logic to user account store
        let account_address = event_body.user;
        let liquidation_collateral_data = AaveAccountMovement::new(
            block_number,
            transaction_hash,
            AaveEventType::LiquidationWithdrawCollateral,
            account_address,
            collateral_token,
            event_body.liquidated_collateral_amount,
            a_token_liquidated_amount,
        );
        let liquidation_debt_data = AaveAccountMovement::new(
            block_number,
            transaction_hash,
            AaveEventType::LiquidationRepayDebt,
            account_address,
            debt_token,
            event_body.debt_to_cover,
            debt_token_amount,
        );
        let user_data = self
            .user_account_store
            .get_user_account(&account_address)
            .await;
        user_data.write().await.liquidate(
            &liquidation_debt_data,
            &liquidation_collateral_data,
        )?;

        Ok(vec![
            liquidation_collateral_data,
            liquidation_debt_data,
        ])
    }

    async fn handle_a_token_transfer(
        &self,
        block_number: u64,
        transaction_hash: TxHash,
        event_body: Erc20TransferEventDto,
    ) -> Result<Vec<AaveAccountMovement>, eyre::Error> {
        let token = token_service::get_token_by_a_token_address(
            &event_body.token_address,
        );

        let a_token_amount = self
            .blockchain_facade
            .get_native_token_valuation(
                token.address,
                event_body.value,
                block_number,
            )
            .await
            .wrap_err("Failed to get aToken amount for transfer event")?;

        info!(
            "Handling ERC20 TRANSFER of a-token-amount: {}; amount {} {} from user: {:#x} to: {:#x}",
            token.get_repr(a_token_amount)?,
            token.get_repr(event_body.value)?,
            token.get_symbol(),
            event_body.from,
            event_body.to,
        );

        let from_user_account = self
            .user_account_store
            .get_user_account(&event_body.from)
            .await;
        let mut user_data_from = from_user_account.write().await;

        // in case of transfer to paraswap adapter this is a collateral swap, which works as
        // withdrawal of collateral asset and then supply of swapped asset in different event
        let is_collateral_swap = event_body.to.address
            == get_global_settings()
                .blockchain
                .aave_v3_paraswap_liquidity_swap_address;

        let event_from = AaveAccountMovement::new(
            block_number,
            transaction_hash,
            if is_collateral_swap {
                AaveEventType::Withdraw
            } else {
                AaveEventType::CollateralTransfer
            },
            event_body.from,
            token,
            event_body.value,
            a_token_amount,
        );
        user_data_from.handle_transfer_out(&event_from)?;

        // in case of transfer to paraswap liquidity swap adapter we have a collateral asset
        // swap and we don't need to add transfered amount to aave swap contract address.
        // later there will be an event for this account with supply of swapped token
        if !is_collateral_swap {
            let event_to = AaveAccountMovement::new(
                block_number,
                transaction_hash,
                AaveEventType::CollateralTransfer,
                event_body.to,
                token,
                event_body.value,
                a_token_amount,
            );
            let user_data_to = self
                .user_account_store
                .get_user_account(&event_body.to)
                .await;
            let mut to_user_data = user_data_to.write().await;
            to_user_data.handle_transfer_in(&event_to)?;
        }

        Ok(vec![event_from])
    }
}
