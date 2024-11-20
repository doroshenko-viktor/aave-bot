mod blocks;
mod config;
mod contracts;
mod events;
mod persistence;
mod utils;
use std::{collections::HashSet, time::Duration};

use alloy::{
    providers::{Provider, ProviderBuilder},
    rpc::types::Log,
    sol_types::SolEvent,
    transports::ws::WsConnect,
};
use contracts::pool::Pool::{self, Borrow};
use dotenv::dotenv;
use eyre::{eyre, Error, Result};
use persistence::{
    blocks::{BlockDbId, BlockDto, BlocksPersistence},
    borrow::BorrowEventDto,
};
use sqlx::{postgres::PgPoolOptions, types::BigDecimal};
use tokio::time;

const FIRST_BORROW_EVENT_BLOCK: u64 = 16496939;

#[derive(Debug, Default)]
struct EventGroups {
    block_numbers: HashSet<u64>,
    borrows: Vec<BorrowEventDto>,
    errors: Vec<Error>,
}

impl EventGroups {
    fn add_block_number(&mut self, block_number: u64) {
        self.block_numbers.insert(block_number);
    }

    pub fn add_log(&mut self, cur: Log) {
        match cur.block_number {
            Some(block_number) => {
                self.add_block_number(block_number);
            }
            None => {
                self.errors.push(eyre!("Block does not have block number"));
                return;
            }
        };
        let event_signature = cur.topics().first().unwrap();
        match *event_signature {
            Pool::Borrow::SIGNATURE_HASH => {
                self.add_borrow_event_log(cur);
            }
            Pool::Repay::SIGNATURE_HASH => {
                // self.add_borrow_event_log(cur);
            }
            Pool::Withdraw::SIGNATURE_HASH => {
                // self.add_borrow_event_log(cur);
            }
            Pool::LiquidationCall::SIGNATURE_HASH => {
                // self.add_borrow_event_log(cur);
            }
            _ => {
                self.errors
                    .push(eyre!("Can't parse Log - not supported signature"));
            }
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
                println!("BORROW EVENT: {:#?}", event);
                self.borrows.push(event)
            }
            Err(e) => self.errors.push(e),
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    dotenv().ok();

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&std::env::var("DATABASE_URL")?)
        .await?;

    let ws_connect = WsConnect::new(config::eth::PROVIDER_URL);
    let provider = ProviderBuilder::new().on_ws(ws_connect).await?;

    let current_block_number = provider.get_block_number().await?;

    let index_step = 10;
    let mut index_low = FIRST_BORROW_EVENT_BLOCK;
    let mut index_high = index_low + index_step;

    for _ in 0..current_block_number {
        time::sleep(Duration::from_millis(100)).await;
        println!("CURRENT HIGH INDEX: {}", index_high);
        let logs = events::get_event_logs(&provider, index_low, index_high).await?;

        index_low = index_high + 1;
        index_high += index_step;

        let no_logs = logs.is_empty();
        if no_logs {
            println!("No events in this block");
            continue;
        }
        let block_ids = logs
            .iter()
            .filter_map(|x| x.block_number)
            .collect::<Vec<u64>>();

        let events = logs
            .into_iter()
            .fold(EventGroups::default(), |mut acc, cur| {
                acc.add_log(cur);
                acc
            });

        let EventGroups {
            block_numbers,
            borrows,
            errors,
        } = events;

        let blocks =
            blocks::get_blocks(&provider, &block_numbers.into_iter().collect::<Vec<u64>>())
                .await
                .iter()
                .filter_map(|block| BlockDto::try_from(block).ok())
                .collect::<Vec<BlockDto>>();

        let ids = pool.save_blocks(blocks).await?;
        let borrows = borrows.into_iter().fold(
            Vec::<(BlockDbId, BorrowEventDto)>::new(),
            |mut acc, borrow_event| {
                let id = ids
                    .get(&BigDecimal::from(borrow_event.block_number))
                    .inspect(|block_db_id| {
                        acc.push((*block_db_id.clone(), borrow_event));
                    });
                acc
            },
        );
        if !borrows.is_empty() {
            persistence::borrow::save_borrow_events(&pool, borrows).await?;
        }

        if !errors.is_empty() {
            for error in errors {
                eprintln!("{}", error)
            }
        }

        if !no_logs {
            break; //only for test
        }
    }

    Ok(())
}
