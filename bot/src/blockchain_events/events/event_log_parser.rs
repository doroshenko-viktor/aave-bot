use alloy::{rpc::types::Log, sol_types::SolEvent};

use crate::{
    blockchain_events::{
        borrow::BorrowEventDto, liquidation::LiquidationEventDto, repay::RepayEventDto,
        supply::SupplyEventDto, withdraw::WithdrawEventDto,
    },
    contracts::pool::Pool,
};
use eyre::{eyre, Error, Result};

use super::BlockchainEvent;

pub fn parse_event_logs(logs: &[Log]) -> (Vec<BlockchainEvent>, Vec<Error>) {
    logs.iter()
        .map(|log| {
            let event = parse_event_log(log)?;
            Ok(event)
        })
        .fold(
            (Vec::<BlockchainEvent>::new(), Vec::<Error>::new()),
            |mut acc, cur| {
                match cur {
                    Ok(event) => acc.0.push(event),
                    Err(err) => acc.1.push(err),
                }
                acc
            },
        )
}

pub fn parse_event_log(log: &Log) -> Result<BlockchainEvent> {
    let event_signature = log.topics().first().unwrap();
    match *event_signature {
        Pool::Supply::SIGNATURE_HASH => {
            let event = log.log_decode::<Pool::Supply>()?;
            let event = SupplyEventDto::try_from(&event)?;
            Ok(BlockchainEvent::Supply(event))
        }
        Pool::Borrow::SIGNATURE_HASH => {
            let event = log.log_decode::<Pool::Borrow>()?;
            let event = BorrowEventDto::try_from(&event)?;
            Ok(BlockchainEvent::Borrow(event))
        }
        Pool::Repay::SIGNATURE_HASH => {
            let event = log.log_decode::<Pool::Repay>()?;
            let event = RepayEventDto::try_from(&event)?;
            Ok(BlockchainEvent::Repay(event))
        }
        Pool::Withdraw::SIGNATURE_HASH => {
            let event = log.log_decode::<Pool::Withdraw>()?;
            let event = WithdrawEventDto::try_from(&event)?;
            Ok(BlockchainEvent::Withdraw(event))
        }
        Pool::LiquidationCall::SIGNATURE_HASH => {
            let event = log.log_decode::<Pool::LiquidationCall>()?;
            let event = LiquidationEventDto::try_from(&event)?;
            Ok(BlockchainEvent::Liquidation(event))
        }
        _ => Err(eyre!("Can't parse Log - not supported signature")),
    }
}
