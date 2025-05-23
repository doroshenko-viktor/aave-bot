use super::{
    borrow::BorrowEventDto, liquidation::LiquidationEventDto,
    repay::RepayEventDto, supply::SupplyEventDto, withdraw::WithdrawEventDto,
};
use crate::{
    contracts::pool::Pool,
    tokens::{
        TokenAddress,
        blockchain::{
            contracts::IERC20, erc20_transfer::Erc20TransferEventDto,
        },
    },
};
use alloy::{hex, primitives::TxHash, rpc::types::Log, sol_types::SolEvent};
use eyre::{Error, Ok, OptionExt, eyre};
use serde::{Deserialize, Serialize};

pub mod event_log_parser;
pub mod get;
pub mod save;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BlockchainEvent {
    pub transaction_hash: TxHash,
    pub block_number: u64,
    pub log_number: u64,
    pub body: BlockchainEventBody,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum BlockchainEventBody {
    Supply(SupplyEventDto),
    Borrow(BorrowEventDto),
    Repay(RepayEventDto),
    Withdraw(WithdrawEventDto),
    Liquidation(LiquidationEventDto),
    Erc20Transfer(Erc20TransferEventDto),
}

pub struct EventTokens {
    pub collaterals: Vec<TokenAddress>,
    pub debts: Vec<TokenAddress>,
}

impl EventTokens {
    pub fn new(
        collaterals: Vec<TokenAddress>,
        debts: Vec<TokenAddress>,
    ) -> Self {
        Self { collaterals, debts }
    }
}

impl BlockchainEventBody {
    pub fn get_tokens(&self) -> EventTokens {
        let tokens = match self {
            BlockchainEventBody::Supply(supply_event_dto) => {
                EventTokens::new(vec![supply_event_dto.reserve], vec![])
            }
            BlockchainEventBody::Borrow(borrow_event_dto) => {
                EventTokens::new(vec![], vec![borrow_event_dto.reserve])
            }
            BlockchainEventBody::Repay(repay_event_dto) => {
                EventTokens::new(vec![], vec![repay_event_dto.reserve])
            }
            BlockchainEventBody::Withdraw(withdraw_event_dto) => {
                EventTokens::new(vec![withdraw_event_dto.reserve], vec![])
            }
            BlockchainEventBody::Liquidation(liquidation_event_dto) => {
                EventTokens::new(
                    vec![liquidation_event_dto.collateral_asset],
                    vec![liquidation_event_dto.debt_asset],
                )
            }
            BlockchainEventBody::Erc20Transfer(erc20_transfer_event_dto) => {
                EventTokens::new(
                    vec![erc20_transfer_event_dto.token_address],
                    vec![],
                )
            }
        };
        tokens
    }
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
            IERC20::Transfer::SIGNATURE_HASH => {
                let event = log.log_decode::<IERC20::Transfer>()?;
                BlockchainEventBody::Erc20Transfer(
                    Erc20TransferEventDto::try_from(&event)?,
                )
            }
            _ => {
                return Err(eyre!(
                    "Can't parse Log - not supported signature"
                ));
            }
        };
        let transaction_hash = log.transaction_hash.ok_or_eyre(
            "Blockchain event transaction hash must be not empty",
        )?;

        Ok(BlockchainEvent {
            transaction_hash,
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
    pub transaction_hash: String,
    pub block_number: i64,
    pub log_number: i32,
    pub event_type_id: i32,
    pub body: serde_json::Value,
}

impl TryInto<BlockchainEvent> for BlockchainEventPersistenceDto {
    type Error = Error;

    fn try_into(self) -> Result<BlockchainEvent, Self::Error> {
        let transaction_hash = TxHash::from_slice(
            &hex::decode(&self.transaction_hash)
                .map_err(|_| eyre!("Can't decode transaction hash"))?,
        );
        let block_number = self.block_number as u64;
        let log_number = self.log_number as u64;
        let body: BlockchainEventBody = self.try_into()?;

        Ok(
            BlockchainEvent {
                transaction_hash,
                block_number,
                log_number,
                body,
            },
        )
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
            6 => BlockchainEventBody::Erc20Transfer(serde_json::from_value(
                self.body,
            )?),
            _ => {
                return Err(eyre!(
                    "Can't parse BlockchainEventPersistenceDto - not supported event type id"
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
            BlockchainEventBody::Erc20Transfer(inner) => {
                (6, serde_json::to_value(inner)?)
            }
        };
        let transaction_hash = hex::encode(value.transaction_hash);

        Ok(BlockchainEventPersistenceDto {
            transaction_hash,
            block_number: value.block_number as i64,
            log_number: value.log_number as i32,
            event_type_id,
            body,
        })
    }
}
