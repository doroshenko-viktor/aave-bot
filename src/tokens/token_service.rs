use super::types::TokenId;
use super::{TokenAddress, TokenSymbol, entities::Token, persistence};
use crate::shared::persistence::DbPool;
use hashbrown::HashMap;
use std::{collections::HashSet, sync::OnceLock};
use tracing::debug;

// TODO: use AsyncOnceCell or OnceCell
static TOKENS: OnceLock<HashMap<TokenId, Token>> = OnceLock::new();
static TOKENS_BY_ADDRESSES: OnceLock<HashMap<&TokenAddress, &Token>> =
    OnceLock::new();
static TOKENS_BY_SYMBOLS: OnceLock<HashMap<&TokenSymbol, &Token>> =
    OnceLock::new();
static TOKENS_BY_A_TOKEN_ADDRESSES: OnceLock<HashMap<&TokenAddress, &Token>> =
    OnceLock::new();

pub async fn setup_tokens(db_pool: &DbPool) -> eyre::Result<()> {
    let _ = TOKENS.set(
        persistence::get_tokens(db_pool)
            .await
            .expect("Tokens configuration must be read correctly")
            .into_iter()
            .fold(HashMap::new(), |mut res, token| {
                res.insert(token.id, token);
                res
            }),
    );

    TOKENS_BY_ADDRESSES.set(
        TOKENS
            .get()
            .expect("TOKENS accessed before set")
            .iter()
            .fold(HashMap::new(), |mut res, (_, cur)| {
                res.insert(&cur.address, cur);
                res
            }),
    );

    TOKENS_BY_SYMBOLS.set(
        TOKENS
            .get()
            .expect("TOKENS accessed before set")
            .iter()
            .fold(HashMap::new(), |mut res, (_, cur)| {
                res.insert(&cur.token_symbol, cur);
                res
            }),
    );

    TOKENS_BY_A_TOKEN_ADDRESSES
        .set(
            TOKENS
                .get()
                .expect("TOKENS accessed before set")
                .iter()
                .fold(HashMap::new(), |mut res, (_, cur)| {
                    res.insert(&cur.a_token_address, cur);
                    res
                }),
        )
        .expect("Failed to set TOKENS_BY_A_TOKEN_ADDRESSES");

    Ok(())
}

pub fn get_tokens() -> Vec<&'static Token> {
    TOKENS
        .get()
        .expect("Tokens are misconfigured in token_service")
        .values()
        .collect()
}

pub fn get_token_by_id(token_id: TokenId) -> &'static Token {
    TOKENS
        .get()
        .expect("TokenId is misconfigured in token_service")
        .get(&token_id)
        .expect("TokenId is misconfigured in token_service")
}

pub fn get_token_by_address(address: &TokenAddress) -> &'static Token {
    debug!("get_token_by_address: {:?}", address);
    TOKENS_BY_ADDRESSES
        .get()
        .unwrap()
        .get(address)
        .map(|data| data)
        .expect("TokenAddress is misconfigured in token_service")
}

pub fn get_token_by_a_token_address(address: &TokenAddress) -> &'static Token {
    debug!(
        "get_token_by_a_token_address: {:?}",
        address
    );
    TOKENS_BY_A_TOKEN_ADDRESSES
        .get()
        .unwrap()
        .get(address)
        .map(|data| data)
        .expect("ATokenAddress is misconfigured in token_service")
}

pub fn get_tokens_by_addresses(
    addresses: HashSet<&TokenAddress>,
) -> Vec<&'static Token> {
    let tokens = TOKENS_BY_ADDRESSES.get().unwrap();
    addresses
        .into_iter()
        .filter_map(|address| tokens.get(address).map(|data| *data))
        .collect()
}

pub fn get_token_by_symbol(token_symbol: &TokenSymbol) -> &'static Token {
    TOKENS_BY_SYMBOLS
        .get()
        .unwrap()
        .get(token_symbol)
        .map(|x| x)
        .expect("TokenSymbol is misconfigured in token_service")
}
