use std::{any::Any, error::Error};

use crate::{contracts::chainlink::Chainlink, PubSubNodeProvider};
use alloy::{
    primitives::Address,
    providers::Provider,
    pubsub::{Subscription, SubscriptionStream},
    rpc::types::{Filter, Log},
    sol_types::SolEvent,
};
use eyre::Result;
use futures::StreamExt;
use tokio::sync::{broadcast, mpsc::Receiver};
use tracing::error;

pub struct Config {
    pub aggregator_address: Address,
}

pub async fn subscribe_price_feed_logs(
    provider: &PubSubNodeProvider,
    config: Config,
) -> Result<SubscriptionStream<Log>> {
    let filter = Filter::new().address(config.aggregator_address);
    // .event_signature(Chainlink::AnswerUpdated::SIGNATURE_HASH);
    let log_stream = provider.subscribe_logs(&filter).await?.into_stream();

    Ok(log_stream)

    // let (tx, rx) = tokio::sync::mpsc::channel::<Log>(1000);

    // tokio::spawn(async move {
    //     loop {
    //         match log_stream.recv().await {
    //             Ok(log) => {
    //                 // println!("New log received: {:#?}", log);
    //                 if let Err(e) = tx.send(log).await {
    //                     error!("ERROR SENDING IN CHANNEL:{}", e)
    //                 }
    //             }
    //             Err(err) => {
    //                 // match err {
    //                 //     tokio::sync::broadcast::error::RecvError::Closed => todo!(),
    //                 //     tokio::sync::broadcast::error::RecvError::Lagged(_) => todo!(),
    //                 // }
    //                 error!("Error receiving log: {:?}", err.to_string());
    //                 break;
    //             }
    //         }
    //     }
    // });

    // Ok(rx)
}
