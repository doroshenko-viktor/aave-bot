use crate::{
    accounts::types::AssetAmount,
    blockchain_events::{
        borrow::BorrowEventDto, liquidation::LiquidationEventDto,
        repay::RepayEventDto, supply::SupplyEventDto,
        withdraw::WithdrawEventDto,
    },
    configuration::globals,
    price_feed::prices_aggregator,
    shared::persistence::AccountAddress,
    tokens::{token_service, types::TokenId},
};
use alloy::primitives::Address;
use eyre::{Result, eyre};
use hashbrown::{HashMap, hash_map::Entry};
use std::{collections::HashSet, sync::Arc};
use tracing::{debug, warn};

pub type UserAccountRef = Arc<tokio::sync::RwLock<UserAccount>>;

#[derive(Debug, Default)]
pub struct UserAccount {
    /* Account address on blockchain */
    pub(crate) address: Address,
    // TODO: change to be EnumMap for better CPU caching
    // or [Option<_>; N] where N is amount of tokens configured and Some for asset present in the
    // system
    // pub(crate) collaterals: [Option<AssetAmount>; 32],
    // pub(crate) debts: [Option<AssetAmount>; 32],
    pub(crate) collaterals: HashMap<TokenId, AssetAmount>,
    pub(crate) debts: HashMap<TokenId, AssetAmount>,
}

impl UserAccount {
    pub fn new(
        address: Address,
        collaterals: HashMap<TokenId, AssetAmount>,
        debts: HashMap<TokenId, AssetAmount>,
    ) -> Self {
        Self { address, collaterals, debts }
    }

    pub fn get_address(&self) -> AccountAddress {
        self.address
    }

    // TODO: maybe maintain set of related tokens on account and change it on blockchain events, because it happens not as often as price change events to reduce compute time
    pub fn get_related_tokens(&self) -> HashSet<TokenId> {
        self.collaterals
            .iter()
            .map(|(token_id, _)| *token_id)
            .chain(self.debts.iter().map(|(token_id, _)| *token_id))
            .collect()
    }

    /**
     * Health Factor= ∑(Collateral Amount×Collateral Price×Liquidation Threshold)/ ∑(Borrowed Amount×Borrowed Asset Price)
     */
    // TODO: this calculation uses thresold for both collaterals and debts, but should only for
    // collaterals
    pub async fn calculate_health_factor(&self) -> Result<f32> {
        Ok(
            self.get_asset_sum(&self.collaterals).await?
                / self.get_asset_sum(&self.debts).await?,
        )
    }

    async fn get_asset_sum(
        &self,
        list: &HashMap<TokenId, AssetAmount>,
    ) -> Result<f32> {
        let asset_rules = globals::get_asset_rules();
        let health_factor = list
            .iter()
            .filter_map(|(token_id, asset_amount)| {
                let token = token_service::get_token_by_id(*token_id);
                let threshold =
                    asset_rules.get_asset_liquidation_threshold(*token_id);
                let Some(price) = prices_aggregator::get_usd_price(*token_id)
                else {
                    warn!(
                        "Price for token {} not found",
                        token.get_symbol()
                    );

                    return None;
                };

                // TODO:need to divide or asset amount or price for decimals because price is not
                // for wei
                Some((asset_amount, price, threshold))
            })
            .fold(
                0f32,
                |mut res, (asset_amount, price, threshold)| {
                    // TODO: make proper calculations with tests
                    // res += price * asset_amount * threshold;
                    res
                },
            );

        Ok(health_factor)
    }

    pub fn get_collateral_list(
        &self,
    ) -> impl Iterator<Item = (TokenId, AssetAmount)> {
        self.collaterals
            .iter()
            .map(|(token_id, amount)| (*token_id, *amount))
    }

    pub fn get_debt_list(
        &self,
    ) -> impl Iterator<Item = (TokenId, AssetAmount)> {
        self.debts
            .iter()
            .map(|(token_id, amount)| (*token_id, *amount))
    }

    pub fn handle_supply(&mut self, event: SupplyEventDto) {
        let reserve_token = token_service::get_token_by_address(&event.reserve);
        self.collaterals
            .entry(reserve_token.get_id())
            .and_modify(|val| {
                debug!(
                    "Updating collateral amount {}=>{}",
                    val, event.amount
                );
                *val += &event.amount;
            })
            .or_insert_with(|| {
                debug!(
                    "Adding collateral with init amount {}",
                    event.amount
                );
                event.amount
            });
    }

    pub fn handle_borrow(&mut self, event: BorrowEventDto) {
        let reserve_token = token_service::get_token_by_address(&event.reserve);
        self.debts
            .entry(reserve_token.get_id())
            .and_modify(|val| {
                debug!(
                    "Updating debt amount {}=>{}",
                    val, event.amount
                );
                *val += &event.amount;
            })
            .or_insert_with(|| {
                debug!(
                    "Adding debt with init amount {}",
                    event.amount
                );
                event.amount
            });
    }

    pub fn handle_withdraw(&mut self, event: WithdrawEventDto) -> Result<()> {
        let reserve_token = token_service::get_token_by_address(&event.reserve);
        match self.collaterals.entry(reserve_token.get_id()) {
            Entry::Occupied(mut val) => {
                let val = val.get_mut();
                *val -= &event.amount;
                debug!("Updated collateral amount {}", val);
                Ok(())
            }
            Entry::Vacant(_) => Err(eyre!(
                "Collateral with for asset '{}' not found",
                event.reserve
            )),
        }
    }

    pub fn handle_repay(&mut self, event: RepayEventDto) -> Result<()> {
        let reserve_token = token_service::get_token_by_address(&event.reserve);
        let asset =
            if let Some(asset) = self.debts.get_mut(&reserve_token.get_id()) {
                asset
            } else {
                return Err(eyre!(
                    "Not found debt asset to be repaid"
                ));
            };

        debug!("\tdebt asset before: {}", asset);
        debug!("\trepay event amount: {}", event.amount);
        *asset = *asset - event.amount;
        debug!("\tdebt asset after: {}", asset);

        Ok(())
    }

    pub fn handle_liquidation(
        &mut self,
        event: LiquidationEventDto,
    ) -> Result<()> {
        let debt_token = token_service::get_token_by_address(&event.debt_asset);
        let debt_asset_amount = if let Some(debt_asset) =
            self.debts.get_mut(&debt_token.get_id())
        {
            debt_asset
        } else {
            return Err(eyre!(
                "Not found debt asset to be repaid by liquidation"
            ));
        };

        let collateral_token =
            token_service::get_token_by_address(&event.collateral_asset);
        let collateral_asset_amount = if let Some(collateral_asset) =
            self.collaterals.get_mut(&collateral_token.get_id())
        {
            collateral_asset
        } else {
            return Err(eyre!(
                "Not found collateral asset to be liquidated"
            ));
        };

        debug!(
            "debt asset before: {}",
            debt_asset_amount
        );
        debug!(
            "debt to cover amount: {}",
            event.debt_to_cover
        );
        *debt_asset_amount = *debt_asset_amount - event.debt_to_cover;
        debug!(
            "debt asset after: {}",
            debt_asset_amount
        );
        debug!(
            "collateral asset before: {}",
            collateral_asset_amount
        );
        debug!(
            "liquidated collateral amount: {}",
            event.liquidated_collateral_amount
        );
        *collateral_asset_amount =
            *collateral_asset_amount - event.liquidated_collateral_amount;
        debug!(
            "collateral asset after: {}",
            collateral_asset_amount
        );

        Ok(())
    }

    pub fn address(&self) -> Address {
        self.address
    }
}

impl std::hash::Hash for UserAccount {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.address.hash(state);
    }
}

impl PartialEq for UserAccount {
    fn eq(&self, other: &Self) -> bool {
        self.address == other.address
    }
}
