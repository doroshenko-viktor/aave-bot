use alloy::{providers::RootProvider, pubsub::PubSubFrontend};
pub mod app;
pub mod clients;
pub mod configuration;
pub mod contracts;
pub mod persistence;
pub mod setup;
pub mod utils;
mod lib_tests;

pub type NodeProvider = RootProvider<PubSubFrontend>;