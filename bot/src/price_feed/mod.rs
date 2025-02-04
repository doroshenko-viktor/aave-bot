pub mod rpc_client;
use alloy::{
    json_abi::JsonAbi, primitives::Address, providers::Provider, rpc::types::Filter,
    sol_types::SolEvent,
};
use eyre::Result;

use crate::{contracts::chainlink::ChainlinkAggregator, PubSubNodeProvider};

const latest_round_data_abi: &str = r#"[{
        "inputs": [],
        "name": "latestRoundData",
        "outputs": [
            {"internalType": "uint80", "name": "roundId", "type": "uint80"},
            {"internalType": "int256", "name": "answer", "type": "int256"},
            {"internalType": "uint256", "name": "startedAt", "type": "uint256"},
            {"internalType": "uint256", "name": "updatedAt", "type": "uint256"},
            {"internalType": "uint80", "name": "answeredInRound", "type": "uint80"}
        ],
        "stateMutability": "view",
        "type": "function"
    }]"#;
