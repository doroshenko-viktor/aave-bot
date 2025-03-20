use crate::{
    accounts::value_objects::AccountAsset,
    blockchain_events::{
        borrow::BorrowEventDto, liquidation::LiquidationEventDto, repay::RepayEventDto,
        supply::SupplyEventDto, withdraw::WithdrawEventDto,
    },
    shared::persistence::AccountAddress,
    tokens::{entities::Token, token_service},
};
use alloy::primitives::Address;
use eyre::{Result, eyre};
use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};
use tracing::debug;

use super::AssetAmount;

pub type UserAccountRef = Arc<tokio::sync::RwLock<UserAccount>>;

#[derive(Debug, Default)]
pub struct UserAccount {
    /* Account address on blockchain */
    pub(crate) address: Address,
    pub(crate) collaterals: HashMap<&'static Token, AccountAsset>,
    pub(crate) debts: HashMap<&'static Token, AccountAsset>,
}

impl UserAccount {
    pub fn new(
        address: Address,
        collaterals: HashMap<&'static Token, AccountAsset>,
        debts: HashMap<&'static Token, AccountAsset>,
    ) -> Self {
        Self {
            address,
            collaterals,
            debts,
        }
    }

    pub fn get_address(&self) -> AccountAddress {
        self.address
    }

    // TODO: maybe maintain set of related tokens on account and change it on blockchain events, because it happens not as often as price change events to reduce compute time
    pub fn get_related_tokens(&self) -> HashSet<&'static Token> {
        self.collaterals
            .keys()
            .chain(self.debts.keys())
            .copied()
            .collect()
    }

    pub fn get_collateral_list(&self) -> Vec<(&'static Token, AssetAmount)> {
        self.collaterals
            .iter()
            .map(|(token, asset)| (*token, asset.amount.clone()))
            .collect()
    }

    pub fn get_debt_list(&self) -> Vec<(&'static Token, AssetAmount)> {
        self.debts
            .iter()
            .map(|(token, asset)| (*token, asset.amount.clone()))
            .collect()
    }

    pub fn handle_supply(&mut self, event: SupplyEventDto) {
        let reserve_token = token_service::get_token(&event.reserve);
        self.collaterals
            .entry(reserve_token)
            .and_modify(|val| {
                debug!(
                    "Updating collateral amount {}=>{}",
                    val.amount, event.amount
                );
                val.amount += &event.amount;
            })
            .or_insert_with(|| {
                debug!("Adding collateral with init amount {}", event.amount);
                AccountAsset::new(event.amount)
            });
    }

    pub fn handle_borrow(&mut self, event: BorrowEventDto) {
        let reserve_token = token_service::get_token(&event.reserve);
        self.debts
            .entry(reserve_token)
            .and_modify(|val| {
                debug!("Updating debt amount {}=>{}", val.amount, event.amount);
                val.amount += &event.amount;
            })
            .or_insert_with(|| {
                debug!("Adding debt with init amount {}", event.amount);
                AccountAsset::new(event.amount)
            });
    }

    pub fn handle_withdraw(&mut self, event: WithdrawEventDto) -> Result<()> {
        let reserve_token = token_service::get_token(&event.reserve);
        match self.collaterals.entry(reserve_token) {
            std::collections::hash_map::Entry::Occupied(mut val) => {
                let val = val.get_mut();
                val.amount -= &event.amount;
                debug!("Updated collateral amount {}", val.amount);
                Ok(())
            }
            std::collections::hash_map::Entry::Vacant(_) => Err(eyre!(
                "Collateral with for asset '{}' not found",
                event.reserve
            )),
        }
    }

    pub fn handle_repay(&mut self, event: RepayEventDto) -> Result<()> {
        let reserve_token = token_service::get_token(&event.reserve);
        let asset = if let Some(asset) = self.debts.get_mut(reserve_token) {
            asset
        } else {
            return Err(eyre!("Not found debt asset to be repaid"));
        };

        debug!("\tdebt asset before: {}", asset.amount);
        debug!("\trepay event amount: {}", event.amount);
        asset.amount = &asset.amount - &event.amount;
        debug!("\tdebt asset after: {}", asset.amount);

        Ok(())
    }

    pub fn handle_liquidation(&mut self, event: LiquidationEventDto) -> Result<()> {
        let debt_token = token_service::get_token(&event.debt_asset);
        let debt_asset = if let Some(debt_asset) = self.debts.get_mut(debt_token) {
            debt_asset
        } else {
            return Err(eyre!("Not found debt asset to be repaid by liquidation"));
        };

        let collateral_token = token_service::get_token(&event.collateral_asset);
        let collateral_asset =
            if let Some(collateral_asset) = self.collaterals.get_mut(collateral_token) {
                collateral_asset
            } else {
                return Err(eyre!("Not found collateral asset to be liquidated"));
            };

        debug!("debt asset before: {}", debt_asset.amount);
        debug!("debt to cover amount: {}", event.debt_to_cover);
        debt_asset.amount = &debt_asset.amount - &event.debt_to_cover;
        debug!("debt asset after: {}", debt_asset.amount);
        debug!("collateral asset before: {}", collateral_asset.amount);
        debug!(
            "liquidated collateral amount: {}",
            event.liquidated_collateral_amount
        );
        collateral_asset.amount = &collateral_asset.amount - &event.liquidated_collateral_amount;
        debug!("collateral asset after: {}", collateral_asset.amount);

        Ok(())
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
