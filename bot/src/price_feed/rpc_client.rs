use crate::{contracts::chainlink::ChainlinkAggregator, PubSubNodeProvider};
use alloy::{
    primitives::Address,
    providers::Provider,
    rpc::types::{Filter, Log},
    sol_types::SolEvent,
};
use eyre::Result;
use tokio::sync::mpsc::Receiver;

pub struct Config {
    pub aggregator_address: Address,
}

pub async fn subscribe_price_feed_logs(
    provider: &PubSubNodeProvider,
    config: Config,
) -> Result<Receiver<Log>> {
    let filter = Filter::new()
        .address(config.aggregator_address)
        .event_signature(ChainlinkAggregator::AnswerUpdated::SIGNATURE_HASH);
    let mut log_stream = provider.subscribe_logs(&filter).await?;

    let (tx, rx) = tokio::sync::mpsc::channel::<Log>(100);

    tokio::spawn(async move {
        loop {
            match log_stream.recv().await {
                Ok(log) => {
                    println!("New log received: {:#?}", log);
                    tx.send(log);
                }
                Err(err) => {
                    eprintln!("Error receiving log: {:?}", err);
                    break;
                }
            }
        }
    });

    Ok(rx)
}
