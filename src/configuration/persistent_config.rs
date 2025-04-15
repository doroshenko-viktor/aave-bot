use dashmap::DashMap;
use eyre::{Result, eyre};
use sqlx::{query, query_scalar};
use std::{
    fmt::Display,
    sync::{Arc, LazyLock},
};

use super::globals;

static CONFIGURATION_CACHE: LazyLock<Arc<DashMap<ConfigurationKey, String>>> =
    LazyLock::new(|| Arc::new(DashMap::new()));

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
            ConfigurationKey::LastDownloadedBlock => {
                write!(f, "last_downloaded_block")
            }
            ConfigurationKey::LastUserBalancesBlock => {
                write!(f, "last_user_balances_block")
            }
        }
    }
}

pub async fn config_set(key: ConfigurationKey, value: &str) -> Result<()> {
    let pool = globals::get_db_pool();

    CONFIGURATION_CACHE.insert(key, value.to_string());

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

pub async fn get_as<T>(key: ConfigurationKey) -> Result<T>
where
    T: std::str::FromStr,
{
    let value = config_get(key).await?;
    value
        .parse::<T>()
        .map_err(|_| eyre!("Failed to parse value: {}", value))
}

pub async fn config_get(key: ConfigurationKey) -> Result<String> {
    match CONFIGURATION_CACHE.get(&key) {
        Some(value) => Ok(value.clone()),
        None => {
            let pool = globals::get_db_pool();
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
