use crate::{shared::persistence::DbPool, tokens::TokenSymbol};
use eyre::Result;
use sqlx;

use super::globals::get_db_pool;

//TODO: check if I need it
// pub async fn get_tokens_to_track() -> Result<Vec<TokenSymbol>> {
//     let pool = get_db_pool();
//     let res: Vec<String> = sqlx::query!(
//         r#"
//         SELECT token FROM config.active_tokens
//         WHERE is_active = true
//         "#,
//     )
//     .fetch_all(pool)
//     .await?
//     .into_iter()
//     .filter_map(|row| row.token)
//     .collect();

//     Ok(res)
// }
