use hashbrown::HashMap;

use super::types::TokenId;
use super::{TokenAddress, TokenSymbol, entities::Token, persistence};
use std::{collections::HashSet, sync::OnceLock};

// TODO: use AsyncOnceCell or OnceCell
pub static TOKENS: OnceLock<HashMap<TokenId, Token>> = OnceLock::new();

pub async fn setup_tokens() -> eyre::Result<()> {
    let tokens = persistence::get_tokens()
        .await
        .expect("Tokens configuration must be read correctly")
        .into_iter()
        .fold(HashMap::new(), |mut res, (token, _)| {
            res.insert(token.id, token);

            res
        });
    let _ = TOKENS.set(tokens);

    Ok(())
}

pub static TOKENS_BY_ADDRESSES: OnceLock<HashMap<&TokenAddress, &Token>> =
    OnceLock::new();
// LazyLock::new(|| {
//     TOKENS.iter().fold(HashMap::new(), |mut res, cur| {
//         res.insert(&cur.1.address, cur.1);
//         res
//     })
// });

pub static TOKENS_BY_SYMBOLS: OnceLock<HashMap<&TokenSymbol, &Token>> =
    OnceLock::new();
// LazyLock::new(|| {
//     TOKENS.iter().fold(HashMap::new(), |mut res, cur| {
//         res.insert(&cur.1.token_symbol, cur.1);
//         res
//     })
// });

pub fn get_token_by_id(token_id: TokenId) -> &'static Token {
    TOKENS
        .get()
        .expect("TokenId is misconfigured in token_service")
        .get(&token_id)
        .expect("TokenId is misconfigured in token_service")
}

pub fn get_token_by_address(address: &TokenAddress) -> &'static Token {
    TOKENS_BY_ADDRESSES
        .get()
        .unwrap()
        .get(address)
        .map(|data| data)
        .expect("TokenAddress is misconfigured in token_service")
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
