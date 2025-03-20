use std::{collections::HashMap, sync::LazyLock};

use crate::tokens::entities::Token;

use super::{
    entities::{TickerData, TickerId, TickerSymbol},
    persistence,
};

struct TickerInternal {
    pub ticker_id: TickerId,
    pub ticker_data: TickerData,
}

/**
* This service serves to get a static data about Ticker.
* **This service normally is not thread safe, but it should be only used as &'static reference, which will be tread safe for concurrent reads**
*/

/* Main index from ticker(which is represented only by id) to it's full data */
static TICKERS: LazyLock<HashMap<TickerId, TickerData>> = LazyLock::new(|| {
    let runtime = tokio::runtime::Runtime::new().unwrap();
    let (tickers, _) = runtime
        .block_on(async { persistence::get_active_tickers().await })
        .expect("Tickers configuration must be read correctly")
        .into_iter()
        .fold((HashMap::new(), 0), |mut res, ticker_data| {
            let ticker_id = TickerId::new(res.1);
            res.0.insert(ticker_id, ticker_data);
            res.1 += 1;

            res
        });

    tickers
});

/* maps bybit ticker representation to bybit representation */
static BYBIT_INVERSE_MAP: LazyLock<HashMap<TickerSymbol, TickerInternal>> = LazyLock::new(|| {
    TICKERS
        .iter()
        .fold(HashMap::new(), |mut res, (ticker_id, ticker_data)| {
            res.insert(
                ticker_data.bybit_repr.clone(),
                TickerInternal {
                    ticker_id: ticker_id.clone(),
                    ticker_data: ticker_data.clone(),
                },
            );
            res
        })
});

pub fn get_active_tickers() -> Vec<(TickerId, TickerData)> {
    TICKERS
        .iter()
        .map(|(key, value)| (key.clone(), value.clone()))
        .collect()
}

pub fn get_ticker_data(ticker: &TickerId) -> &'_ TickerData {
    TICKERS
        .get(ticker)
        .expect("Ticker has to be in configuration")
}

pub fn get_tickers_data(tickers: &[TickerId]) -> impl Iterator<Item = &'_ TickerData> {
    tickers.iter().map(|ticker| {
        TICKERS
            .get(ticker)
            .expect("Ticker has to be in configuration")
    })
}

pub fn get_ticker_from_bybit_str<'a>(
    bybit_ticker: &'a str,
) -> (&'static TickerId, &'static TickerData) {
    let x = BYBIT_INVERSE_MAP
        .get(bybit_ticker)
        .map(|x| (&x.ticker_id, &x.ticker_data))
        .expect("Bybit ticker str name is misconfigured");
    x
}

pub fn get_main_token_from_bybit_str<'a>(bybit_ticker: &'a str) -> &'static Token {
    let x = BYBIT_INVERSE_MAP
        .get(bybit_ticker)
        .map(|x| &x.ticker_data.token_1)
        .expect("Bybit ticker str name is misconfigured");
    x
}
