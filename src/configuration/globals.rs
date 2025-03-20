use super::Settings;
use crate::{
    NodeWsProvider, shared::persistence::DbPool, tokens::asset_rules_aggregator::AssetRules,
};
use tokio::sync::OnceCell;

/*
These global variables should be static and not change it state over application lifetime or do it in some straightforward way
*/
pub static DB_POOL: OnceCell<DbPool> = OnceCell::const_new();
pub static RPC_PROVIDER: OnceCell<NodeWsProvider> = OnceCell::const_new();
pub static SETTINGS: OnceCell<Settings> = OnceCell::const_new();
pub static ASSET_RULES: OnceCell<AssetRules> = OnceCell::const_new();

/**
 * Returns DB_POOL assuming it is already initialized on app startup.
 * If it called before initialization this will panic.
 */
pub fn get_db_pool() -> &'static DbPool {
    return DB_POOL
        .get()
        .expect("DB_POOL requested before initialization");
}

pub fn get_global_settings() -> &'static Settings {
    return SETTINGS
        .get()
        .expect("Settings requested before initialization");
}

pub fn get_rpc_provider() -> &'static NodeWsProvider {
    return RPC_PROVIDER
        .get()
        .expect("RPC_PROVIDER requested before initialization");
}

pub fn get_asset_rules() -> &'static AssetRules {
    return ASSET_RULES
        .get()
        .expect("ASSET_RULES requested before initialization");
}
