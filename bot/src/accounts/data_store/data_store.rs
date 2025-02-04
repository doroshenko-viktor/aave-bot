use std::{cell::Cell, collections::HashMap};

use alloy::hex;
use tracing::debug;

use crate::blockchain_events::{
    borrow::BorrowEventDto, events::BlockchainEvent, liquidation::LiquidationEventDto,
    repay::RepayEventDto, supply::SupplyEventDto, withdraw::WithdrawEventDto,
};

use super::value_objects::UserData;

const TEST: bool = true;

#[derive(Default)]
pub struct UserDataStore {
    pub data: HashMap<Vec<u8>, UserData>,
    pub last_handled_block_number: Cell<u64>,
}

impl UserDataStore {
    pub fn get_last_block_number(&self) -> u64 {
        self.last_handled_block_number.get()
    }
    fn set_last_block_number(&self, block_number: u64) {
        self.last_handled_block_number.replace(block_number);
    }

    pub fn handle_events(&mut self, events: &[BlockchainEvent]) {
        for event in events {
            self.handle_event(event);
        }
    }

    pub fn handle_event(&mut self, event: &BlockchainEvent) {
        let x = hex::decode("523B27014B865C4D8F3DAC9C6E3B1BAD13D22B87").unwrap();
        let test_user = hex::encode(x);
        match event {
            BlockchainEvent::Supply(event) => {
                if TEST
                    && (hex::encode(event.user.clone()) != test_user
                        && hex::encode(event.on_behalf_of.clone()) != test_user)
                {
                    return;
                }
                self.handle_supply_event(event);
            }
            BlockchainEvent::Borrow(event) => {
                if TEST
                    && (hex::encode(event.user.clone()) != test_user
                        && hex::encode(event.on_behalf_of.clone()) != test_user)
                {
                    return;
                }
                self.handle_borrow_event(&event);
            }
            BlockchainEvent::Repay(event) => {
                if TEST && hex::encode(event.user.clone()) != test_user {
                    return;
                }
                self.handle_repay_event(event);
            }
            BlockchainEvent::Withdraw(event) => {
                if TEST && hex::encode(event.user.clone()) != test_user {
                    return;
                }
                self.handle_withdraw_event(event);
            }
            BlockchainEvent::Liquidation(event) => {
                if TEST && hex::encode(event.user.clone()) != test_user {
                    return;
                }
                self.handle_liquidation_event(event);
            }
        }
    }

    fn handle_supply_event(&mut self, event: &SupplyEventDto) {
        debug!("Handle supply");
        let on_behalf_of = hex::encode(event.on_behalf_of.clone());
        let user = hex::encode(event.user.clone());
        debug!("\tOn behalf of: {}; user: {}", on_behalf_of, user);
        let user_data = if user != on_behalf_of {
            self.data.entry(event.on_behalf_of.clone()).or_default()
        } else {
            self.data.entry(event.user.clone()).or_default()
        };
        user_data.handle_supply(event);
        self.set_last_block_number(event.block_number);
    }

    fn handle_borrow_event(&mut self, event: &BorrowEventDto) {
        debug!("Handle borrow");

        let on_behalf_of = hex::encode(event.on_behalf_of.clone());
        let user = hex::encode(event.user.clone());
        debug!("\tOn behalf of: {}; user: {}", on_behalf_of, user);
        let user_data = if event.user != event.on_behalf_of {
            self.data.entry(event.on_behalf_of.clone()).or_default()
        } else {
            self.data.entry(event.user.clone()).or_default()
        };
        user_data.handle_borrow(&event);
        self.set_last_block_number(event.block_number);
    }

    fn handle_liquidation_event(&mut self, event: &LiquidationEventDto) {
        debug!("Handle liquidation");
        let user_data = self.data.entry(event.user.clone()).or_default();
        user_data.handle_liquidation(&event);
        self.set_last_block_number(event.block_number);
    }

    fn handle_withdraw_event(&mut self, event: &WithdrawEventDto) {
        debug!("Handle withdraw");
        let user_data = self.data.entry(event.user.clone()).or_default();
        user_data.handle_withdraw(&event);
        self.set_last_block_number(event.block_number);
    }

    fn handle_repay_event(&mut self, event: &RepayEventDto) {
        debug!("Handle repay");
        let user_data = self.data.entry(event.user.clone()).or_default();
        user_data.handle_repay(&event);
        self.set_last_block_number(event.block_number);
    }
}
