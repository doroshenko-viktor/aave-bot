use crate::{app::entities::ERC20TokenEntity, shared::persistence::DbPool};
use eyre::Result;
use sqlx::query_as;

pub async fn get_tokens(pool: &DbPool) -> Result<Vec<ERC20TokenEntity>> {
    let tokens = query_as!(
        ERC20TokenEntity,
        r#"
        select
            id,
            address,
            name,
            symbol,
            decimals,
            total_supply
        from blockchain.tokens;
        "#
    )
    .fetch_all(pool)
    .await?;

    Ok(tokens)
}
