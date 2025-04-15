use std::str::FromStr;

use crate::{
    shared::persistence::DbPool,
    tokens::{ERC20TokenDbDto, entities::Token},
};
use alloy::primitives::{Address, Uint};
use eyre::Result;
use tracing::error;

pub async fn get_tokens(
    db_pool: &DbPool,
) -> Result<impl Iterator<Item = Token>> {
    let tokens = sqlx::query_as!(
        ERC20TokenDbDto,
        r#"
        SELECT
            id,
            address,
            name,
            symbol,
            decimals,
            total_supply,
            a_token_address,
            stable_debt_token_address,
            variable_debt_token_address
        FROM tokens.tokens;
        "#
    )
    .fetch_all(db_pool)
    .await?
    .into_iter()
    .map(|rec| {
        let token_address = Address::from_slice(&rec.address);
        let x = rec.total_supply.with_scale(0).to_string();
        println!("x: {}", x);
        let total_supply = match Uint::<256, 4>::from_str(&x) {
            Ok(val) => val,
            Err(e) => {
                error!("Failed to parse total supply: {}", e);
                panic!("Failed to parse total supply")
            }
        };
        Token::new(
            rec.id as u8,
            rec.symbol,
            token_address,
            rec.decimals as u8,
            total_supply,
        )
    });

    Ok(tokens)
}
