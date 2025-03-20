# aave-bot

rayon
async streams

## Links

[Smart contracts docs](https://aave.com/docs/developers/smart-contracts)
[Ethers-Rs Docs](https://www.gakonst.com/ethers-rs/)

## Onchain Data

Repay event - first block 16497341
16498089; // withdraw event block

## TODO:

- move lib types to shared
- prices feed emits price events - consider mechanism to track event ids and skip handling of newer emmitted events for this thicker if generated more events than handled, e.g. keep track of ticker event ids in prices aggregator
- refactor contracts folder
- add clear config cache endpoint
- create function which will get current block and log numbers(meaning last computed event) and will start to fetch events from database if it contains more events than given block number, then will fetch from infura if no events in db until current block and then will use only realtime events
  this function will call compute function for user account balances and in another thread will save new events to db

## Notes

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
