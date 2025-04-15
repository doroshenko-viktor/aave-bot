use std::str::FromStr;

use crate::{
    shared::persistence::{DbPool, get_placeholders_str},
    tokens::{ERC20TokenDbDto, ERC20TokenDto, ERC20TokenSaveDbDto},
};
use eyre::Result;
use sqlx::types::BigDecimal;
use tracing::{debug, info};

pub async fn save_tokens(
    pool: &DbPool,
    tokens: Vec<ERC20TokenSaveDbDto>,
) -> Result<()> {
    debug!(
        "Saving {} tokens to database",
        tokens.len()
    );
    let mut query_builder = r#"INSERT INTO tokens.tokens (
            address,
            name,
            symbol,
            decimals,
            total_supply,
            a_token_address,
            stable_debt_token_address,
            variable_debt_token_address
        ) VALUES"#
        .to_string();

    query_builder.push_str(&get_placeholders_str(8, tokens.len()));
    query_builder.push_str(r#" ON CONFLICT (address) DO NOTHING"#);
    let mut query = sqlx::query(&query_builder);

    for token in tokens {
        let total_supply =
            BigDecimal::from_str(&token.total_supply.to_string())?;

        query = query
            .bind(token.address.to_vec())
            .bind(token.name)
            .bind(token.symbol)
            .bind(token.decimals)
            .bind(total_supply)
            .bind(token.a_token_address.to_vec())
            .bind(token.stable_debt_token_address.to_vec())
            .bind(token.variable_debt_token_address.to_vec());
    }
    let res = query.execute(pool).await?;
    info!("INSERTION RES: {:#?}", res);

    Ok(())
}
