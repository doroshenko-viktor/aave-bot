use crate::clients::events as events_client;
use crate::configuration::AppConfig;
use crate::contracts::pool::Pool::{self, Borrow, LiquidationCall, Repay, Supply, Withdraw};
use crate::persistence::borrow::BorrowEventDto;
use crate::persistence::configuration::{self, ConfigurationKey};
use crate::persistence::liquidation::LiquidationEventDto;
use crate::persistence::repay::RepayEventDto;
use crate::persistence::supply::{SupplyEventDto, SupplyEventsPersistence};
use crate::persistence::withdraw::WithdrawEventDto;
use crate::NodeProvider;
use std::time::Duration;

use crate::persistence::{
    borrow::BorrowEventsPersistence, liquidation::LiquidationEventsPersistence,
    repay::RepayEventsPersistence, withdraw::WithdrawEventsPersistence, PostgresPool,
};
use alloy::providers::Provider;
use alloy::rpc::types::Log;
use alloy::sol_types::SolEvent;
use eyre::{eyre, Error, Result};
use futures::future::join_all;
use tokio::time;
use tracing::{debug, error, info};

pub async fn fetch_events(
    provider: &NodeProvider,
    pool: &PostgresPool,
    config: &AppConfig,
) -> Result<()> {
    let current_block_number = provider.get_block_number().await?;
    let mut index_low = config.first_block;
    let mut index_high = index_low + config.indexing_step;

    for _ in 0..current_block_number {
        time::sleep(Duration::from_millis(100)).await;
        info!("CURRENT HIGH INDEX: {}", index_high);
        let logs = events_client::get_event_logs(provider, index_low, index_high).await?;
        debug!("Retrieved logs amount: {}", logs.len());

        index_low = index_high + 1;
        index_high += config.indexing_step;

        let no_logs = logs.is_empty();
        if no_logs {
            debug!("No events in this block");
            continue;
        }

        let events = logs
            .into_iter()
            .fold(EventGroups::default(), |mut acc, cur| {
                acc.add_log(cur);
                acc
            });

        let EventGroups {
            supplies,
            borrows,
            errors,
            repays,
            liquidations,
            withdrawals,
        } = events;

        info!(
            "Borrows: {}; Repays: {}, Withdrawals: {}, Liquidations: {}",
            borrows.len(),
            repays.len(),
            withdrawals.len(),
            liquidations.len()
        );

        let mut futures = Vec::new();

        if !supplies.is_empty() {
            futures.push(pool.save_supply_events(supplies));
        }
        if !borrows.is_empty() {
            futures.push(pool.save_borrow_events(borrows));
        }
        if !repays.is_empty() {
            futures.push(pool.save_repay_events(repays));
        }
        if !withdrawals.is_empty() {
            futures.push(pool.save_withdraw_events(withdrawals));
        }
        if !liquidations.is_empty() {
            futures.push(pool.save_liquidation_events(liquidations));
        }

        let results = join_all(futures).await;
        for result in results {
            if let Err(err) = result {
                error!("Error inserting events to db: {:#?}", err);
            }
        }

        if !errors.is_empty() {
            for error in errors {
                error!("{}", error)
            }
        }

        configuration::set(
            pool,
            ConfigurationKey::LastHandledBlock,
            &index_high.to_string(),
        )
        .await?;
    }

    Ok(())
}

#[derive(Debug, Default)]
struct EventGroups {
    supplies: Vec<SupplyEventDto>,
    borrows: Vec<BorrowEventDto>,
    repays: Vec<RepayEventDto>,
    liquidations: Vec<LiquidationEventDto>,
    withdrawals: Vec<WithdrawEventDto>,
    errors: Vec<Error>,
}

impl EventGroups {
    pub fn add_log(&mut self, cur: Log) {
        let event_signature = cur.topics().first().unwrap();
        match *event_signature {
            Pool::Supply::SIGNATURE_HASH => {
                self.add_supply_event_log(cur);
            }
            Pool::Borrow::SIGNATURE_HASH => {
                self.add_borrow_event_log(cur);
            }
            Pool::Repay::SIGNATURE_HASH => {
                self.add_repay_event_log(cur);
            }
            Pool::Withdraw::SIGNATURE_HASH => {
                self.add_withdraw_event_log(cur);
            }
            Pool::LiquidationCall::SIGNATURE_HASH => {
                self.add_liquidation_event_log(cur);
            }
            _ => {
                self.errors
                    .push(eyre!("Can't parse Log - not supported signature"));
            }
        }
    }

    // TODO: consider generic code here - too similar code
    pub fn add_supply_event_log(&mut self, event: Log) {
        fn parse(
            event: Result<Log<Supply>, alloy::sol_types::Error>,
        ) -> Result<SupplyEventDto, Error> {
            let event = event?;
            let event = SupplyEventDto::try_from(&event)?;
            Ok(event)
        }
        let event = event.log_decode::<Pool::Supply>();
        match parse(event) {
            Ok(event) => {
                info!("Added LIQUIDATION EVENT: {:#?}", event);
                self.supplies.push(event)
            }
            Err(e) => self.errors.push(e),
        }
    }
    pub fn add_liquidation_event_log(&mut self, event: Log) {
        fn parse(
            event: Result<Log<LiquidationCall>, alloy::sol_types::Error>,
        ) -> Result<LiquidationEventDto, Error> {
            let event = event?;
            let event = LiquidationEventDto::try_from(&event)?;
            Ok(event)
        }
        let event = event.log_decode::<Pool::LiquidationCall>();
        match parse(event) {
            Ok(event) => {
                info!("Added LIQUIDATION EVENT: {:#?}", event);
                self.liquidations.push(event)
            }
            Err(e) => self.errors.push(e),
        }
    }

    pub fn add_withdraw_event_log(&mut self, event: Log) {
        fn parse(
            event: Result<Log<Withdraw>, alloy::sol_types::Error>,
        ) -> Result<WithdrawEventDto, Error> {
            let event = event?;
            let event = WithdrawEventDto::try_from(&event)?;
            Ok(event)
        }
        let event = event.log_decode::<Pool::Withdraw>();
        match parse(event) {
            Ok(event) => {
                info!("Added WITHDRAW EVENT: {:#?}", event);
                self.withdrawals.push(event)
            }
            Err(e) => self.errors.push(e),
        }
    }

    pub fn add_repay_event_log(&mut self, event: Log) {
        fn parse(
            event: Result<Log<Repay>, alloy::sol_types::Error>,
        ) -> Result<RepayEventDto, Error> {
            let event = event?;
            let borrow_event = RepayEventDto::try_from(&event)?;
            Ok(borrow_event)
        }
        let event = event.log_decode::<Pool::Repay>();
        match parse(event) {
            Ok(event) => {
                info!("Added REPAY EVENT: {:#?}", event);
                self.repays.push(event)
            }
            Err(e) => self.errors.push(e),
        }
    }

    pub fn add_borrow_event_log(&mut self, event: Log) {
        fn parse(
            event: Result<Log<Borrow>, alloy::sol_types::Error>,
        ) -> Result<BorrowEventDto, Error> {
            let event = event?;
            let borrow_event = BorrowEventDto::try_from(&event)?;
            Ok(borrow_event)
        }

        let event = event.log_decode::<Pool::Borrow>();
        match parse(event) {
            Ok(event) => {
                self.borrows.push(event);
            }
            Err(e) => self.errors.push(e),
        }
    }
}
