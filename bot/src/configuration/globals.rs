use super::Settings;
use crate::{shared::persistence::DbPool, PubSubNodeProvider};
use sqlx::PgPool;
use tokio::sync::OnceCell;

pub static SETTINGS: OnceCell<Settings> = OnceCell::const_new();
pub static DB_POOL: OnceCell<DbPool> = OnceCell::const_new();
pub static RPC_PROVIDER: OnceCell<PubSubNodeProvider> = OnceCell::const_new();

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

pub fn get_rpc_provider() -> &'static PubSubNodeProvider {
    return RPC_PROVIDER
        .get()
        .expect("RPC_PROVIDER requested before initialization");
}
