use super::ERC20TokenDto;
use crate::shared::persistence::{get_placeholders_str, DbPool};
use eyre::Result;
use tracing::{debug, info};

pub async fn save_tokens(pool: &DbPool, tokens: Vec<ERC20TokenDto>) -> Result<()> {
    debug!("Saving {} tokens to database", tokens.len());
    let mut query_builder = r#"INSERT INTO blockchain.tokens (
            address,
            name,
            symbol,
            decimals,
            total_supply
        ) VALUES"#
        .to_string();

    query_builder.push_str(&get_placeholders_str(5, tokens.len()));
    query_builder.push_str(r#" ON CONFLICT (address) DO NOTHING"#);
    let mut query = sqlx::query(&query_builder);

    for token in tokens {
        query = query
            .bind(token.address)
            .bind(token.name)
            .bind(token.symbol)
            .bind(token.decimals)
            .bind(token.total_supply);
    }
    let res = query.execute(pool).await?;
    info!("INSERTION RES: {:#?}", res);

    Ok(())
}
