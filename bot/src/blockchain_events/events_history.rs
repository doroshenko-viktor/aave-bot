use crate::clients::events as events_client;
use crate::configuration::persistent_confit::{config_set, ConfigurationKey};
use crate::configuration::{self, PrcRateLimiter};
use crate::contracts::pool::Pool::{self, Borrow, LiquidationCall, Repay, Supply, Withdraw};
use crate::shared::persistence::PostgresPool;
use crate::utils::iterators::GapSequence;
use crate::NodeProvider;
use alloy::providers::Provider;
use alloy::rpc::types::Log;
use alloy::sol_types::SolEvent;
use eyre::{eyre, Error, Result};
use futures::future::join_all;
use futures::{task, FutureExt};
use sqlx::PgPool;
use tracing::{debug, error, info};

pub struct FetchEventsConfig {
    pub first_block_number: u64,
    pub last_block_number: u64,
    /**
     * Defines how many blocks to fetch in single step
     */
    pub indexing_step: u64,
}

/**
 * Fetch and save blockchain events to database
 */
pub async fn fetch_events(
    provider: &NodeProvider,
    pool: &PostgresPool,
    config: FetchEventsConfig,
    rate_limiter: PrcRateLimiter,
) -> Result<()> {
    // let current_block_number = provider.get_block_number().await?;
    // let mut index_low = config.first_block_number;
    // let mut index_high = index_low + config.indexing_step;
    let mut lower_index = 0u64;

    for (lower, higher) in GapSequence::new(
        config.first_block_number,
        config.last_block_number,
        config.indexing_step,
    ) {
        debug!("CURRENT HIGH INDEX: {}", higher);
        rate_limiter.until_ready().await;
        let logs = events_client::get_event_logs(provider, lower, higher).await?;
        debug!("Retrieved logs amount: {}", logs.len());

        config_set(
            ConfigurationKey::LastHandledBlock,
            &higher.to_string(),
        )
        .await?;
    }

    for _ in 0..config.last_block_number {
        let no_logs = logs.is_empty();
        if no_logs {
            debug!("No events in this block range");
            continue;
        }

        let events = logs
            .into_iter()
            .fold(EventGroups::default(), |mut acc, cur| {
                acc.add_log(cur);
                acc
            });

        save_events(pool, events).await?;
    }

    Ok(())
}

async fn save_events(pool: &PgPool, events: EventGroups) -> Result<()> {
    let EventGroups {
        supplies,
        borrows,
        errors,
        repays,
        liquidations,
        withdrawals,
    } = events;

    let mut save_events_futures = Vec::new();

    if !supplies.is_empty() {
        save_events_futures.push(save_supply_events(pool, supplies).boxed());
    }
    if !borrows.is_empty() {
        save_events_futures.push(save_borrow_events(pool, borrows).boxed());
    }
    if !repays.is_empty() {
        save_events_futures.push(save_repay_events(pool, repays).boxed());
    }
    if !withdrawals.is_empty() {
        save_events_futures.push(save_withdraw_events(pool, withdrawals).boxed());
    }
    if !liquidations.is_empty() {
        save_events_futures.push(save_liquidation_events(pool, liquidations).boxed());
    }

    let results = join_all(save_events_futures).await;
    for result in results {
        if let Err(err) = result {
            error!("Error inserting events to db: {:#?}", err);
        }
    }

    if !errors.is_empty() {
        for error in errors {
            save_error(pool, format!("Error during save events history: {}", error)).await?;
        }
    }
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
