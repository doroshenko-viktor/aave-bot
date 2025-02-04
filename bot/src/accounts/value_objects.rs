use std::collections::HashMap;

use alloy::hex;
use bigdecimal::BigDecimal;
use tracing::debug;

use crate::blockchain_events::{
    borrow::BorrowEventDto, events::BlockchainEvent, liquidation::LiquidationEventDto,
    repay::RepayEventDto, supply::SupplyEventDto, withdraw::WithdrawEventDto,
};

use super::types::ReserveAsset;

pub struct Asset {
    reserve: ReserveAsset, // TODO: use token reference instead of raw address
    amount: BigDecimal,
}
impl Asset {
    pub fn new(reserve: Vec<u8>) -> Self {
        Self {
            reserve,
            amount: BigDecimal::from(0),
        }
    }
}

#[derive(Default)]
pub struct UserData {
    collateral: HashMap<ReserveAsset, Asset>,
    debt: HashMap<ReserveAsset, Asset>,
}

impl UserData {
    pub fn handle_event(&mut self, event: &BlockchainEvent) {
        match event {
            BlockchainEvent::Supply(event) => self.handle_supply(event),
            BlockchainEvent::Borrow(event) => self.handle_borrow(event),
            BlockchainEvent::Repay(event) => self.handle_repay(event),
            BlockchainEvent::Withdraw(event) => self.handle_withdraw(event),
            BlockchainEvent::Liquidation(event) => self.handle_liquidation(event),
        }
    }

    pub fn handle_supply(&mut self, event: &SupplyEventDto) {
        let collateral_asset = self
            .collateral
            .entry(event.reserve.clone())
            .or_insert_with(|| Asset::new(event.reserve.clone()));
        let on_behalf_of = hex::encode(&event.on_behalf_of);
        let user = hex::encode(&event.user);
        debug!("\tuser: {}; on behalf of: {}", user, on_behalf_of);
        debug!("collateral asset before: {}", collateral_asset.amount);
        debug!("event amount: {}", event.amount);
        collateral_asset.amount = &collateral_asset.amount + &event.amount;
        debug!("collateral asset after: {}", collateral_asset.amount);
    }

    pub fn handle_borrow(&mut self, event: &BorrowEventDto) {
        let asset = self
            .debt
            .entry(event.reserve.clone())
            .or_insert_with(|| Asset::new(event.reserve.clone()));
        let on_behalf_of = hex::encode(&event.on_behalf_of);
        let user = hex::encode(&event.user);
        debug!("\tuser: {}; on behalf of: {}", user, on_behalf_of);
        debug!("debt asset before: {}", asset.amount);
        debug!("event amount: {}", event.amount);
        asset.amount = &asset.amount + &event.amount;
        debug!("debt asset after: {}", asset.amount);
    }

    pub fn handle_withdraw(&mut self, event: &WithdrawEventDto) {
        let asset = self
            .collateral
            .entry(event.reserve.clone())
            .or_insert_with(|| Asset::new(event.reserve.clone()));
        let to = hex::encode(&event.to);
        let user = hex::encode(&event.user);
        debug!("\tuser: {}; to: {}", user, to);
        debug!("collateral asset before: {}", asset.amount);
        debug!("event amount: {}", event.amount);
        asset.amount = &asset.amount - &event.amount;
        debug!("collateral asset after: {}", asset.amount);
    }

    pub fn handle_repay(&mut self, event: &RepayEventDto) {
        let asset = self
            .debt
            .entry(event.reserve.clone())
            .or_insert_with(|| Asset::new(event.reserve.clone()));

        let repayer = hex::encode(&event.repayer);
        let user = hex::encode(&event.user);
        debug!("\tuser: {}; repayer: {}", user, repayer);
        debug!("\tdebt asset before: {}", asset.amount);
        debug!("\tevent amount: {}", event.amount);
        asset.amount = &asset.amount - &event.amount;
        debug!("\tdebt asset after: {}", asset.amount);
    }

    pub fn handle_liquidation(&mut self, event: &LiquidationEventDto) {
        let debt_asset = self
            .debt
            .entry(event.debt_asset.clone())
            .or_insert_with(|| Asset::new(event.debt_asset.clone()));
        let collateral_asset = self
            .collateral
            .entry(event.collateral_asset.clone())
            .or_insert_with(|| Asset::new(event.collateral_asset.clone()));

        debug!("debt asset before: {}", debt_asset.amount);
        debug!("event amount: {}", event.debt_to_cover);
        debt_asset.amount = &debt_asset.amount - &event.debt_to_cover;
        debug!("debt asset after: {}", debt_asset.amount);
        debug!("collateral asset before: {}", collateral_asset.amount);
        debug!("event amount: {}", event.liquidated_collateral_amount);
        collateral_asset.amount = &collateral_asset.amount - &event.liquidated_collateral_amount;
        debug!("collateral asset after: {}", collateral_asset.amount);
    }
}
