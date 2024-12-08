use eyre::{Ok, OptionExt, Result};
use sqlx::query;
use std::fmt::Display;

use super::PostgresPool;

pub enum ConfigurationKey {
    LastHandledBlock,
}

impl Display for ConfigurationKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigurationKey::LastHandledBlock => write!(f, "last_handled_block"),
        }
    }
}

pub async fn set(pool: &PostgresPool, key: ConfigurationKey, value: &str) -> Result<()> {
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

pub async fn get(pool: &PostgresPool, key: ConfigurationKey) -> Result<String> {
    let key = key.to_string();
    let result = query!(
        r#"
        select value from config.cache
        where key = $1
        "#,
        key
    )
    .fetch_one(pool)
    .await?;

    let result = result
        .value
        .ok_or_eyre("Configuration value can't be None")?;

    Ok(result)
}
