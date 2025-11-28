# AAVE-Bot: Real-Time AAVE Protocol Monitoring System

## Project Overview

AAVE-Bot is a blockchain monitoring and data aggregation system designed to track user interactions with the AAVE lending protocol in real-time. While originally conceived as a bad debt liquidation bot, the project currently focuses on the foundational data gathering and analysis layer.

### Original Vision

This is more of a Rust study project aimed to apply rust on real world blockchain scenario.

The initial goal was to build an **automated liquidation bot** that would:

- Monitor AAVE protocol accounts for unhealthy positions
- Detect accounts approaching liquidation thresholds
- Execute liquidations to capture liquidation bonuses
- Manage risk and profitability of liquidation operations

### Current State

Currently it is a **real-time data monitoring and aggregation platform** that serves as the foundation for any liquidation strategy. The project implements:

- **Real-time Event Monitoring**: Tracks all AAVE protocol activities (supply, borrow, repay, withdraw, liquidations)
- **Account State Management**: Maintains accurate collateral and debt balances for all monitored accounts
- **Health Factor Calculation**: Computes account health factors to assess liquidation risk
- **Multi-Source Price Aggregation**: Integrates price data from Bybit, Chainlink, and Pyth oracles
- **REST API**: Exposes account movements and token data for external consumption
- **Persistent Data Store**: PostgreSQL backend for event history and configuration

### Project Goals

This project serves dual purposes for me:

1. **Educational**: A hands-on learning project to master:
   - Modern Rust application development for real-world scenarios
   - Blockchain interaction and Web3 programming concepts
   - Async/concurrent Rust patterns with Tokio
   - Building scalable, production-grade systems in Rust
   - Working with relational databases and migrations with Rust

2. **Foundation for Liquidation Logic**: Provides a data layer that could be extended with liquidation execution capabilities

### Project Status

**Implementation Stage**: Data gathering and analysis(No actual liquidation)

The liquidation execution layer has not been implemented. While the project has been deprioritized due to other commitments, it represents technical work demonstrating:

- Rust architecture
- async/concurrent patterns
- Real-world blockchain data handling
- Scalable system design

This codebase serves as a reference implementation for building blockchain monitoring systems and can be extended with liquidation logic or adapted for other DeFi monitoring use cases.

**Note:** You may find some secrets in git history. I understand that this is not suitable for production - currently all of them are invalid and can't be used - this was just a little speed boos on init stage of the project which was private. At this moment there is a configuration which accepts env variables in place.

---

## Technical Stack

**Backend**: Rust with Tokio (async runtime), Axum (web framework), Alloy (Ethereum interaction)
**Database**: PostgreSQL with sqlx migrations
**Frontend**: React 19, Vite, Chakra UI
**Price Feeds**: WebSocket integration with Bybit, Chainlink, and Pyth

---

## Run

To run application in specific environment, use the following env variable:

```bash
export APP_MODE=[dev|prod|stage|test]
```

Then run the application:

```bash
cargo watch -q -x "run --bin web_api --features dev"
```

Or run specific binary:

```bash
cargo run --bin precalculate_user_balances<or other from ./src/bin/>
```

## Links

[Smart contracts docs](https://aave.com/docs/developers/smart-contracts)
[Ethers-Rs Docs](https://www.gakonst.com/ethers-rs/)

## TODO

- create mechanism to get transactions from blockchain if I don't have it and batch download and provide transactions to code when it's historical
- TODO: handle useATokens in repay - burst corresponding aToken amount
- // TODO: move token
  // reference to event_body and find token on event deserialization
- change transaction hash in db to be string
- allow instant negative balances because it may be caused by invalid log order and correct balances are forced by aave anyway
- save account state daily and recaluclate it starting last day's state by historic day events to keep valid state event if events were received not in right order
- track ReserveDataUpdated
- eliminate BigDecimal - use u64 and alloy int<256, 4> for performance
- move lib types to shared
- prices feed emits price events - consider mechanism to track event ids and skip handling of newer emmitted events for this thicker if generated more events than handled, e.g. keep track of ticker event ids in prices aggregator
- refactor contracts folder
- add clear config cache endpoint
- create function which will get current block and log numbers(meaning last computed event) and will start to fetch events from database if it contains more events than given block number, then will fetch from infura if no events in db until current block and then will use only realtime events
  this function will call compute function for user account balances and in another thread will save new events to db

## Notes

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

need to collect user balances, which will be updated by

- blockchain events
- price movements

when there is an update of balance or price need to calculate health factor.
recalculate HF on each blockchain update for individual related account.
but calculating health factors of all accounts on price movements will be inefficient, so need to keep index of most relevant accounts to recalculate HF on price updates.
in separate thread recalculate all accounts HF with current prices and if HF moves to risk zone add them to index of risky accounts

if account's HF goes to liquidation zone check HF request this account's liquidation.
if liquidation was not successful, check why: it may be possible that price from my feeds does not correspond to Chainlink price - then need to start to track not published chainlink data from transactions on blockchain(question when to stop tracking? - maybe when price goes up over certain threshold)

## Alloy tips

- add `#[sol(rpc)]` to sol! macro to generate callable methods for contract.

## Testing

`cargo llvm-cov --open` - to open coverage report in browser

## AAVE

[Deployed Contracts](https://aave.com/docs/resources/addresses)
