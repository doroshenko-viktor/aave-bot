# aave-bot

rayon
async streams

## Run

To run application in specific environment, use the following env variable:

```bash
export APP_MODE=[dev|prod|stage|test]
```

Then run the application:

```bash
cargo watch -q -x "run --bin web_api --features dev"
```

## Links

[Smart contracts docs](https://aave.com/docs/developers/smart-contracts)
[Ethers-Rs Docs](https://www.gakonst.com/ethers-rs/)

## Onchain Data

16496817
Repay event - first block 16497341
16498089; // withdraw event block

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
