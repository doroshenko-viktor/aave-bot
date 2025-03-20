use Chainlink::ChainlinkInstance;
use alloy::{pubsub::PubSubFrontend, sol};

use crate::NodeWsProvider;

sol!(
    #[sol(rpc)]
    Chainlink,
    "abi/chainlink_aggregator.json"
);

pub type ChainlinkInterface = ChainlinkInstance<(), NodeWsProvider>;
