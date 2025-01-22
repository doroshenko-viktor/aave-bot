use crate::NodeProvider;

use super::Settings;
use sqlx::PgPool;
use tokio::sync::OnceCell;

pub static SETTINGS: OnceCell<Settings> = OnceCell::const_new();
pub static DB_POOL: OnceCell<PgPool> = OnceCell::const_new();
pub static RPC_PROVIDER: OnceCell<NodeProvider> = OnceCell::const_new();
