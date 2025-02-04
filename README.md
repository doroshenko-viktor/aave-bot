# aave-bot

## Links

[Smart contracts docs](https://aave.com/docs/developers/smart-contracts)
[Ethers-Rs Docs](https://www.gakonst.com/ethers-rs/)

## Onchain Data

Repay event - first block 16497341
16498089; // withdraw event block

## TODO:

- add clear config cache endpoint
- create function which will get current block and log numbers(meaning last computed event) and will start to fetch events from database if it contains more events than given block number, then will fetch from infura if no events in db until current block and then will use only realtime events
this function will call compute function for user account balances and in another thread will save new events to db