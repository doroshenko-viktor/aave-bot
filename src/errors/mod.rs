use crate::{
    configuration::globals::get_db_pool,
    utils::datetime::chrono_to_primitive_datetime,
};

use chrono::Utc;
use eyre::Result;
use sqlx::query;

pub async fn save_error(message: String) -> Result<()> {
    let timestamp = chrono_to_primitive_datetime(Utc::now());
    let pool = get_db_pool();

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
