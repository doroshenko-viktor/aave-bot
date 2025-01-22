use crate::{
    app::entities::ERC20TokenEntity,
    shared::persistence::{get_placeholders_str, Address, PostgresPool},
};
use bigdecimal::BigDecimal;
use eyre::Result;
use sqlx::query_as;
use tracing::{debug, info};

#[derive(Debug, sqlx::FromRow)]
pub struct ERC20TokenDto {
    pub address: Address,
    pub name: Option<String>,
    pub symbol: String,
    pub decimals: i32,
    pub total_supply: BigDecimal,
}

pub async fn get_tokens(pool: &PostgresPool) -> Result<Vec<ERC20TokenEntity>> {
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

pub async fn save_tokens(pool: &PostgresPool, tokens: Vec<ERC20TokenDto>) -> Result<()> {
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
