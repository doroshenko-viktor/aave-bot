pub mod parser;
use std::sync::Arc;

use crate::{
    NodeWsProvider,
    contracts::chainlink::{Chainlink, ChainlinkInterface},
};
use alloy::{
    consensus::Transaction,
    primitives::{Address, address},
    providers::Provider,
};
use bigdecimal::BigDecimal;
use eyre::Result;
use futures::StreamExt;
use time::PrimitiveDateTime;
use tracing::{error, info};

pub struct ChainlinkFeedDto {
    /* Current feed value */
    pub current: BigDecimal,
    /* A number identifying the data round (or update cycle) in which this value was produced */
    pub round_id: BigDecimal,
    pub updated_at: PrimitiveDateTime,
}

pub async fn get_price(
    feed_client: &ChainlinkInterface,
) -> Result<Chainlink::latestRoundDataReturn> {
    let latest_round_data = feed_client.latestRoundData().call().await?;

    Ok(latest_round_data)
}

pub async fn get_pending_chainlink_transactions_stream(
    provider: &NodeWsProvider,
    chainlink_interface: Arc<ChainlinkInterface>,
    feed_address: Address,
) -> Result<tokio::sync::mpsc::Receiver<Chainlink::latestRoundDataReturn>> {
    let mut pending_transactions = provider
        .subscribe_full_pending_transactions()
        .await?
        .into_stream();

    let (tx, rx) = tokio::sync::mpsc::channel::<Chainlink::latestRoundDataReturn>(100);
    tokio::spawn(async move {
        while let Some(transaction) = pending_transactions.next().await {
            let to = match transaction.to() {
                Some(to) => to,
                None => continue,
            };
            if to == feed_address {
                match get_price(&chainlink_interface).await {
                    Ok(res) => {
                        if let Err(e) = tx.send(res).await {
                            error!(
                                "Error sending chainlink latestRoundDataResult to the channel: {}",
                                e.to_string()
                            );
                        }
                    }
                    Err(e) => error!(
                        "Error receiving latest round data from chainlink: {}",
                        e.to_string()
                    ),
                };
            }
        }
    });

    Ok(rx)
}
