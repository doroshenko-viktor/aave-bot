use alloy::providers::{
    Identity, RootProvider,
    fillers::{
        BlobGasFiller, ChainIdFiller, FillProvider, GasFiller, JoinFill,
        NonceFiller,
    },
};
pub mod accounts;
pub mod app;
pub mod blockchain;
pub mod blockchain_events;
pub mod clients;
pub mod configuration;
pub mod contracts;
pub mod errors;
pub mod price_feed;
pub mod setup;
pub mod shared;
pub mod tokens;
pub mod utils;

pub type NodeWsProvider = FillProvider<
    JoinFill<
        Identity,
        JoinFill<
            GasFiller,
            JoinFill<BlobGasFiller, JoinFill<NonceFiller, ChainIdFiller>>,
        >,
    >,
    RootProvider,
>;

pub type NodeHttpProvider = FillProvider<
    JoinFill<
        Identity,
        JoinFill<
            GasFiller,
            JoinFill<BlobGasFiller, JoinFill<NonceFiller, ChainIdFiller>>,
        >,
    >,
    RootProvider,
>;
