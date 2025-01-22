use eyre::Result;
use sqlx::query;
use std::fmt::Display;

use crate::shared::persistence::PostgresPool;

pub enum ConfigurationKey {
    /* last downloaded blockchain data block */
    LastHandledBlock,
    /* last calculated user balances block */
    LastUserBalancesBlock,
}

impl Display for ConfigurationKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigurationKey::LastHandledBlock => write!(f, "last_handled_block"),
            ConfigurationKey::LastUserBalancesBlock => write!(f, "last_user_balances_block"),
        }
    }
}

pub async fn config_set(key: ConfigurationKey, value: &str) -> Result<()> {
    pool: &PostgresPool, 
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

pub async fn config_get(pool: &PostgresPool, key: ConfigurationKey) -> Result<String> {
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
