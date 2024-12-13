use crate::utils::chrono_to_primitive_datetime;

use super::PostgresPool;
use chrono::Utc;
use eyre::Result;
use sqlx::query;

pub async fn save_error(pool: &PostgresPool, message: String) -> Result<()> {
    let timestamp = chrono_to_primitive_datetime(Utc::now());

    let _ = query!(
        r#"
        INSERT INTO logs.errors ("timestamp", message)
        VALUES ($1, $2)
        "#,
        timestamp,
        message
    )
    .execute(pool)
    .await?;

    Ok(())
}
