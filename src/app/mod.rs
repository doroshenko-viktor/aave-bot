pub mod config;
pub mod startup;
mod tasks;
use crate::{
    accounts::UserAccountStore, blockchain::BlockchainDataProviderFacade,
};
use std::sync::Arc;

// Common type aliases used thoughout application
pub type UserAccountStoreHttpRef<T: BlockchainDataProviderFacade> =
    Arc<UserAccountStore<T>>;
