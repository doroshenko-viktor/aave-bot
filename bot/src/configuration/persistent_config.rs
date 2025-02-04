use eyre::{eyre, OptionExt, Result};
use once_cell::sync::Lazy;
use sqlx::{query, query_scalar};
use std::{collections::HashMap, fmt::Display, sync::Arc};
use tokio::sync::RwLock;

use super::globals::get_db_pool;

static CONFIGURATION_CACHE: Lazy<Arc<RwLock<HashMap<ConfigurationKey, String>>>> =
    Lazy::new(|| Arc::new(RwLock::new(HashMap::new())));

#[derive(Debug, PartialEq, Hash, Eq, Clone, Copy)]
pub enum ConfigurationKey {
    /* last downloaded blockchain data block */
    LastDownloadedBlock,
    /* last calculated user balances block */
    LastUserBalancesBlock,
}

impl Display for ConfigurationKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigurationKey::LastDownloadedBlock => write!(f, "last_downloaded_block"),
            ConfigurationKey::LastUserBalancesBlock => write!(f, "last_user_balances_block"),
        }
    }
}

pub async fn config_set(key: ConfigurationKey, value: &str) -> Result<()> {
    let pool = get_db_pool();

    let mut cache = CONFIGURATION_CACHE.write().await;
    cache.insert(key, value.to_string());

    let key = key.to_string();
    let _ = query!(
        r#"
        INSERT INTO config.cache (key, value)
        VALUES ($1, $2)
        ON CONFLICT (key)
        DO UPDATE SET value = EXCLUDED.value
        "#,
        key,
        value
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn config_get(key: ConfigurationKey) -> Result<String> {
    match CONFIGURATION_CACHE.read().await.get(&key) {
        Some(value) => Ok(value.clone()),
        None => {
            let pool = get_db_pool();
            let key = key.to_string();
            query_scalar!(
                r#"
                select value from config.cache
                where key = $1
                "#,
                key
            )
            .fetch_one(pool)
            .await?
            .ok_or_else(|| eyre!("Configuration {} does not exist", key))
        }
    }
}
