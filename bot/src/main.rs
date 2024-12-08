mod app;
mod clients;
mod configuration;
mod contracts;
mod persistence;
mod setup;
mod utils;

use alloy::{providers::RootProvider, pubsub::PubSubFrontend};
use app::events_history::fetch_events;
// use eyre::Result;
use eyre::Error;
use setup::Application;
use tracing::{debug, error, info, warn};

// const FIRST_BORROW_EVENT_BLOCK: u64 = 16496939; // borrow event block
// const FIRST_BLOCK: u64 = 16497729; // repay event block
// const FIRST_BLOCK: u64 = 16498089; // withdraw event block
// const FIRST_BLOCK: u64 = 16521648; // withdraw event block
const FIRST_BLOCK: u64 = 16797203;

pub type NodeProvider = RootProvider<PubSubFrontend>;

#[tokio::main]
async fn main() -> Result<(), Error> {
    info!("starting bot");
    let Application {
        db_pool,
        rpc_provider,
        settings,
    } = setup::configure_app().await?;

    fetch_events(&rpc_provider, &db_pool, &settings.config).await?;

    info!("Program finished execution");

    Ok(())
}
