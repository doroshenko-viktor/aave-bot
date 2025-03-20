use crate::{
    shared::persistence::{DbPool, get_placeholders_str},
    tokens::ERC20TokenDto,
};
use eyre::Result;
use tracing::{debug, info};

pub async fn save_tokens(pool: &DbPool, tokens: Vec<ERC20TokenDto>) -> Result<()> {
    debug!("Saving {} tokens to database", tokens.len());
    let mut query_builder = r#"INSERT INTO tokens.onchain_token_data (
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
            .bind(token.address.to_vec())
            .bind(token.name)
            .bind(token.symbol)
            .bind(token.decimals)
            .bind(token.total_supply);
    }
    let res = query.execute(pool).await?;
    info!("INSERTION RES: {:#?}", res);

    Ok(())
}
