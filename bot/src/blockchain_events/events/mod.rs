use eyre::{eyre, Error};
use serde::{Deserialize, Serialize};

use super::{
    borrow::BorrowEventDto, liquidation::LiquidationEventDto, repay::RepayEventDto,
    supply::SupplyEventDto, withdraw::WithdrawEventDto,
};

pub mod event_log_parser;
pub mod get;
pub mod save;

#[derive(Debug, Serialize, Deserialize)]
pub enum BlockchainEvent {
    Supply(SupplyEventDto),
    Borrow(BorrowEventDto),
    Repay(RepayEventDto),
    Withdraw(WithdrawEventDto),
    Liquidation(LiquidationEventDto),
}

pub trait ToPersistenceDtos {
    fn to_persistence_dtos(&self) -> (Vec<BlockchainEventPersistenceDto>, Vec<Error>);
}

impl ToPersistenceDtos for &[BlockchainEvent] {
    fn to_persistence_dtos(&self) -> (Vec<BlockchainEventPersistenceDto>, Vec<Error>) {
        return self
            .into_iter()
            .map(BlockchainEventPersistenceDto::try_from)
            .fold(
                (
                    Vec::<BlockchainEventPersistenceDto>::new(),
                    Vec::<Error>::new(),
                ),
                |mut acc, cur| {
                    match cur {
                        Ok(data) => acc.0.push(data),
                        Err(error) => acc.1.push(error),
                    }
                    acc
                },
            );
    }
}

#[derive(Debug, sqlx::FromRow)]
pub struct BlockchainEventPersistenceDto {
    pub block_number: i32,
    pub log_number: i32,
    pub event_type_id: i32,
    pub body: serde_json::Value,
}

impl TryInto<BlockchainEvent> for BlockchainEventPersistenceDto {
    type Error = Error;

    fn try_into(self) -> Result<BlockchainEvent, Self::Error> {
        Ok(match self.event_type_id {
            1 => BlockchainEvent::Supply(serde_json::from_value(self.body)?),
            2 => BlockchainEvent::Borrow(serde_json::from_value(self.body)?),
            3 => BlockchainEvent::Repay(serde_json::from_value(self.body)?),
            4 => BlockchainEvent::Withdraw(serde_json::from_value(self.body)?),
            5 => BlockchainEvent::Liquidation(serde_json::from_value(self.body)?),
            _ => return Err(eyre!("Not supported event type: {}", self.event_type_id)),
        })
    }
}

impl TryFrom<&BlockchainEvent> for BlockchainEventPersistenceDto {
    type Error = Error;

    fn try_from(value: &BlockchainEvent) -> Result<Self, Self::Error> {
        let body = serde_json::to_value(value)?;
        let (block_number, log_number, event_type_id) = match value {
            BlockchainEvent::Supply(event) => (event.block_number, event.log_number, 1),
            BlockchainEvent::Borrow(event) => (event.block_number, event.log_number, 2),
            BlockchainEvent::Repay(event) => (event.block_number, event.log_number, 3),
            BlockchainEvent::Withdraw(event) => (event.block_number, event.log_number, 4),
            BlockchainEvent::Liquidation(event) => (event.block_number, event.log_number, 5),
        };
        let block_number = block_number as i32;
        let log_number = log_number as i32;

        Ok(BlockchainEventPersistenceDto {
            block_number,
            log_number,
            event_type_id,
            body,
        })
    }
}
