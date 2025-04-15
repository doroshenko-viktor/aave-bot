use alloy::{rpc::types::Log, sol_types::SolEvent};
use eyre::{Error, Ok, OptionExt, eyre};
use serde::{Deserialize, Serialize};

use crate::contracts::pool::Pool;

use super::{
    borrow::BorrowEventDto, liquidation::LiquidationEventDto,
    repay::RepayEventDto, supply::SupplyEventDto, withdraw::WithdrawEventDto,
};

pub mod event_log_parser;
pub mod get;
pub mod save;

#[derive(Debug, Serialize, Deserialize)]
pub struct BlockchainEvent {
    pub block_number: u64,
    pub log_number: u64,
    pub body: BlockchainEventBody,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum BlockchainEventBody {
    Supply(SupplyEventDto),
    Borrow(BorrowEventDto),
    Repay(RepayEventDto),
    Withdraw(WithdrawEventDto),
    Liquidation(LiquidationEventDto),
}

impl TryFrom<&Log> for BlockchainEvent {
    type Error = Error;

    fn try_from(log: &Log) -> Result<Self, Self::Error> {
        let event_signature = log.topics().first().unwrap();
        let body = match *event_signature {
            Pool::Supply::SIGNATURE_HASH => {
                let event = log.log_decode::<Pool::Supply>()?;
                BlockchainEventBody::Supply(SupplyEventDto::try_from(&event)?)
            }
            Pool::Borrow::SIGNATURE_HASH => {
                let event = log.log_decode::<Pool::Borrow>()?;
                BlockchainEventBody::Borrow(BorrowEventDto::try_from(&event)?)
            }
            Pool::Repay::SIGNATURE_HASH => {
                let event = log.log_decode::<Pool::Repay>()?;
                BlockchainEventBody::Repay(RepayEventDto::try_from(&event)?)
            }
            Pool::Withdraw::SIGNATURE_HASH => {
                let event = log.log_decode::<Pool::Withdraw>()?;
                BlockchainEventBody::Withdraw(WithdrawEventDto::try_from(
                    &event,
                )?)
            }
            Pool::LiquidationCall::SIGNATURE_HASH => {
                let event = log.log_decode::<Pool::LiquidationCall>()?;
                BlockchainEventBody::Liquidation(LiquidationEventDto::try_from(
                    &event,
                )?)
            }
            _ => {
                return Err(eyre!(
                    "Can't parse Log - not supported signature"
                ));
            }
        };

        Ok(BlockchainEvent {
            block_number: log.block_number.ok_or_eyre(
                "Blockchain event block number must be not empty",
            )?,
            log_number: log
                .log_index
                .ok_or_eyre("Blockchain event log number must be not empty")?,
            body,
        })
    }
}

#[derive(Debug, sqlx::FromRow)]
pub struct BlockchainEventPersistenceDto {
    pub block_number: i64,
    pub log_number: i32,
    pub event_type_id: i32,
    pub body: serde_json::Value,
}

impl TryInto<BlockchainEvent> for BlockchainEventPersistenceDto {
    type Error = Error;

    fn try_into(self) -> Result<BlockchainEvent, Self::Error> {
        let block_number = self.block_number as u64;
        let log_number = self.log_number as u64;
        let body: BlockchainEventBody = self.try_into()?;

        Ok(BlockchainEvent { block_number, log_number, body })
    }
}

impl TryInto<BlockchainEventBody> for BlockchainEventPersistenceDto {
    type Error = Error;

    fn try_into(self) -> Result<BlockchainEventBody, Self::Error> {
        Ok(match self.event_type_id {
            1 => {
                BlockchainEventBody::Supply(serde_json::from_value(self.body)?)
            }
            2 => {
                BlockchainEventBody::Borrow(serde_json::from_value(self.body)?)
            }
            3 => BlockchainEventBody::Repay(serde_json::from_value(self.body)?),
            4 => BlockchainEventBody::Withdraw(serde_json::from_value(
                self.body,
            )?),
            5 => BlockchainEventBody::Liquidation(serde_json::from_value(
                self.body,
            )?),
            _ => {
                return Err(eyre!(
                    "Not supported event type: {}",
                    self.event_type_id
                ));
            }
        })
    }
}

impl TryFrom<&BlockchainEvent> for BlockchainEventPersistenceDto {
    type Error = Error;

    fn try_from(value: &BlockchainEvent) -> Result<Self, Self::Error> {
        let (event_type_id, body) = match &value.body {
            BlockchainEventBody::Supply(inner) => {
                (1, serde_json::to_value(inner)?)
            }
            BlockchainEventBody::Borrow(inner) => {
                (2, serde_json::to_value(inner)?)
            }
            BlockchainEventBody::Repay(inner) => {
                (3, serde_json::to_value(inner)?)
            }
            BlockchainEventBody::Withdraw(inner) => {
                (4, serde_json::to_value(inner)?)
            }
            BlockchainEventBody::Liquidation(inner) => {
                (5, serde_json::to_value(inner)?)
            }
        };

        Ok(BlockchainEventPersistenceDto {
            block_number: value.block_number as i64,
            log_number: value.log_number as i32,
            event_type_id,
            body,
        })
    }
}
