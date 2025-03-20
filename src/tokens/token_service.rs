use super::{TokenAddress, TokenSymbol, entities::Token, persistence};
use alloy::primitives::Address;
use std::{
    collections::{HashMap, HashSet},
    sync::LazyLock,
};

struct TokenData {
    address: Address,
}

pub static TOKENS: LazyLock<HashMap<Token, TokenData>> = LazyLock::new(|| {
    let runtime = tokio::runtime::Runtime::new().unwrap();
    let tickers = runtime
        .block_on(async { persistence::get_tokens().await })
        .expect("Tokens configuration must be read correctly")
        .into_iter()
        .fold(HashMap::new(), |mut res, (internal_token, erc_token)| {
            let token_data = TokenData {
                address: erc_token.address,
            };
            res.insert(internal_token, token_data);

            res
        });

    tickers
});

pub static TOKENS_BY_ADDRESSES: LazyLock<HashMap<&TokenAddress, &Token>> = LazyLock::new(|| {
    TOKENS.iter().fold(HashMap::new(), |mut res, cur| {
        res.insert(&cur.1.address, cur.0);
        res
    })
});

pub static TOKENS_BY_SYMBOLS: LazyLock<HashMap<&TokenSymbol, (&Token, &TokenData)>> =
    LazyLock::new(|| {
        TOKENS.iter().fold(HashMap::new(), |mut res, cur| {
            res.insert(&cur.0.token_symbol, cur);
            res
        })
    });

pub fn get_tokens(addresses: HashSet<&TokenAddress>) -> Vec<&'static Token> {
    addresses
        .into_iter()
        .filter_map(|address| TOKENS_BY_ADDRESSES.get(address).map(|data| *data))
        .collect()
}

pub fn get_token_by_symbol(token_symbol: &TokenSymbol) -> &'static Token {
    TOKENS_BY_SYMBOLS
        .get(token_symbol)
        .map(|x| x.0)
        .expect("TokenSymbol is misconfigured in token_service")
}

pub fn get_token_address(token_symbol: &TokenSymbol) -> TokenAddress {
    TOKENS_BY_SYMBOLS
        .get(token_symbol)
        .map(|x| x.1.address)
        .expect("TokenSymbol is misconfigured in token_service")
}

pub fn get_token(address: &TokenAddress) -> &'static Token {
    TOKENS_BY_ADDRESSES
        .get(address)
        .map(|data| data)
        .expect("TokenAddress is misconfigured in token_service")
}

pub fn get_token_symbol_by_address(address: &TokenAddress) -> &'static TokenSymbol {
    TOKENS_BY_ADDRESSES
        .get(address)
        .map(|token| &token.token_symbol)
        .expect("TokenAddress is misconfigured in token_store")
}
