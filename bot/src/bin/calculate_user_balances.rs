use std::{
    cell::{Cell, RefCell},
    collections::HashMap,
    rc::Rc,
};

use alloy::hex;
use bigdecimal::BigDecimal;
use bot::{
    persistence::{
        assets::{get_tokens, ERC20TokenDto},
        borrow::{get_borrows_sorted_by_block, BorrowEventDto},
        configuration::{self, ConfigurationKey},
        liquidation::{get_liquidations_sorted_by_block, LiquidationEventDto},
        repay::{get_repays_sorted_by_block, RepayEventDto},
        supply::{get_supply_events, SupplyEventDto},
        withdraw::{get_withdraw_events, WithdrawEventDto},
        PostgresPool,
    },
    setup::{self, Application},
};
use eyre::Result;
use tracing::{debug, error};

type ReserveAsset = Vec<u8>;
pub enum Event {
    Supply(SupplyEventDto),
    Borrow(BorrowEventDto),
    Repay(RepayEventDto),
    Withdraw(WithdrawEventDto),
    Liquidation(LiquidationEventDto),
}

struct Asset {
    reserve: ReserveAsset,
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
struct UserData {
    collateral: HashMap<ReserveAsset, Asset>,
    debt: HashMap<ReserveAsset, Asset>,
}

impl UserData {
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

#[derive(Default)]
struct UserDataStore {
    pub data: HashMap<Vec<u8>, UserData>,
}

impl UserDataStore {
    pub fn handle_event(&mut self, event: Event) {
        let test_user = "f200cb437bfe0ad696ecf7cb4264192c7034bbe9".to_string();
        let test = false;
        match event {
            Event::Supply(event) => {
                if test
                    && (hex::encode(event.user.clone()) != test_user
                        && hex::encode(event.on_behalf_of.clone()) != test_user)
                {
                    return;
                }
                debug!("Handle supply");
                // let user_data = self.data.entry(event.user.clone()).or_default();
                let on_behalf_of = hex::encode(event.on_behalf_of.clone());
                let user = hex::encode(event.user.clone());
                debug!("\tOn behalf of: {}; user: {}", on_behalf_of, user);
                let user_data = if user != on_behalf_of {
                    self.data.entry(event.on_behalf_of.clone()).or_default()
                } else {
                    self.data.entry(event.user.clone()).or_default()
                };
                user_data.handle_supply(&event);
            }
            Event::Borrow(event) => {
                if test
                    && (hex::encode(event.user.clone()) != test_user
                        && hex::encode(event.on_behalf_of.clone()) != test_user)
                {
                    return;
                }
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
            }
            Event::Repay(event) => {
                if test && hex::encode(event.user.clone()) != test_user {
                    return;
                }
                debug!("Handle repay");
                let user_data = self.data.entry(event.user.clone()).or_default();
                user_data.handle_repay(&event);
            }
            Event::Withdraw(event) => {
                if test && hex::encode(event.user.clone()) != test_user {
                    return;
                }
                debug!("Handle withdraw");
                let user_data = self.data.entry(event.user.clone()).or_default();
                user_data.handle_withdraw(&event);
            }
            Event::Liquidation(event) => {
                if test && hex::encode(event.user.clone()) != test_user {
                    return;
                }
                debug!("Handle liquidation");
                let user_data = self.data.entry(event.user.clone()).or_default();
                user_data.handle_liquidation(&event);
            }
        }
    }
}

#[tokio::main]
pub async fn main() -> Result<()> {
    let Application {
        db_pool,
        rpc_provider,
        settings,
        rpc_rate_limiter,
    } = setup::configure_app().await?;
    let last_handled_block: i32 = configuration::get(&db_pool, ConfigurationKey::LastHandledBlock)
        .await?
        .parse()?;
    let mut user_data_store: UserDataStore = UserDataStore::default();

    let from_block = 1;
    let to_block = last_handled_block;
    let events = get_events(&db_pool, from_block, to_block).await?;
    for event in events {
        user_data_store.handle_event(event);
    }

    let tokens: HashMap<Vec<u8>, ERC20TokenDto> = get_tokens(&db_pool)
        .await?
        .into_iter()
        .map(|token| (token.address.clone(), token))
        .collect();

    let user_count = user_data_store.data.keys().len();
    for (user_address, user_data) in user_data_store.data {
        let collateral_tokens: Vec<(&ERC20TokenDto, Asset)> = user_data
            .collateral
            .into_iter()
            .map(|asset| {
                let token = tokens.get(&asset.1.reserve);
                if token.is_none() {
                    let token_address = hex::encode(asset.0);
                    error!("Token not found: {}", token_address)
                }
                (token.unwrap(), asset.1)
            })
            .collect();
        let debt_tokens: Vec<(&ERC20TokenDto, Asset)> = user_data
            .debt
            .into_iter()
            .map(|asset| {
                let token = tokens.get(&asset.1.reserve);
                if token.is_none() {
                    let token_address = hex::encode(asset.0);
                    error!("Token not found: {}", token_address)
                }
                (token.unwrap(), asset.1)
            })
            .collect();
        let user_address = hex::encode(user_address);

        println!("USER: {}", user_address);
        println!("COLLATERAL:");
        for (token, asset) in collateral_tokens {
            let divisor = 10.0_f64.powi(token.decimals);
            let amount = asset.amount / divisor;
            println!("\t{}: {}", token.symbol, amount);
        }
        println!("DEBT:");
        for (token, asset) in debt_tokens {
            let divisor = 10.0_f64.powi(token.decimals);
            let amount = asset.amount / divisor;
            println!("\t{}: {}", token.symbol, amount);
        }
    }

    println!("Amount of users: {}", user_count);

    Ok(())
}

async fn get_events(db_pool: &PostgresPool, from_block: i32, to_block: i32) -> Result<Vec<Event>> {
    let mut supplies = get_supply_events(&db_pool, from_block, to_block)
        .await?
        .into_iter()
        .peekable();
    let mut borrows = get_borrows_sorted_by_block(&db_pool, from_block, to_block)
        .await?
        .into_iter()
        .peekable();
    let mut repays = get_repays_sorted_by_block(&db_pool, from_block, to_block)
        .await?
        .into_iter()
        .peekable();
    let mut withdrawals = get_withdraw_events(&db_pool, from_block, to_block)
        .await?
        .into_iter()
        .peekable();
    let mut liquidations = get_liquidations_sorted_by_block(&db_pool, from_block, to_block)
        .await?
        .into_iter()
        .peekable();

    let mut events: Vec<Event> = Vec::new();
    for block_number in from_block..to_block {
        while let Some(event) = supplies.peek() {
            if event.block_number > block_number {
                break;
            }
            let event = supplies.next().unwrap();
            events.push(Event::Supply(event));
        }
        while let Some(event) = borrows.peek() {
            if event.block_number > block_number {
                break;
            }
            let event = borrows.next().unwrap();
            events.push(Event::Borrow(event));
        }
        while let Some(event) = repays.peek() {
            if event.block_number > block_number {
                break;
            }
            let event = repays.next().unwrap();
            events.push(Event::Repay(event));
        }
        while let Some(event) = withdrawals.peek() {
            if event.block_number > block_number {
                break;
            }
            let event = withdrawals.next().unwrap();
            events.push(Event::Withdraw(event));
        }
        while let Some(event) = liquidations.peek() {
            if event.block_number > block_number {
                break;
            }
            let event = liquidations.next().unwrap();
            events.push(Event::Liquidation(event));
        }
    }

    Ok(events)
}
