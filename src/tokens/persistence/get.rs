use crate::{
    app::entities::ERC20TokenEntity, configuration::globals::get_db_pool, tokens::entities::Token,
};
use alloy::primitives::Address;
use eyre::Result;

pub async fn get_tokens() -> Result<impl Iterator<Item = (Token, ERC20TokenEntity)>> {
    let pool = get_db_pool();
    let tokens = sqlx::query!(
        r#"
        select
            tokens.id as token_id,
            tokens.symbol as token_symbol,
            onchain.id as onchain_id,
            address,
            name,
            onchain.symbol as onchain_symbol,
            decimals,
            total_supply
        from tokens.token_details as tokens
        join tokens.onchain_token_data as onchain on tokens.id = onchain.token_id;
        "#
    )
    .fetch_all(pool)
    .await?
    .into_iter()
    .map(|rec| {
        let token = Token::new(rec.token_id as u8, rec.token_symbol);
        let erc_token = ERC20TokenEntity {
            address: Address::from_slice(&rec.address),
            decimals: rec.decimals,
            id: rec.onchain_id,
            name: rec.name,
            symbol: rec.onchain_symbol,
            total_supply: rec.total_supply,
        };

        (token, erc_token)
    });

    Ok(tokens)
}
