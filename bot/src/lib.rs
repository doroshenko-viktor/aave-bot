use alloy::{providers::RootProvider, pubsub::PubSubFrontend};
pub mod accounts;
pub mod app;
pub mod blockchain_events;
pub mod blocks;
pub mod clients;
pub mod configuration;
pub mod contracts;
pub mod errors;
pub mod price_feed;
pub mod setup;
pub mod shared;
pub mod tokens;
pub mod utils;

pub type PubSubNodeProvider = RootProvider<PubSubFrontend>;
