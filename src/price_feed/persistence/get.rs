use eyre::Result;
use sqlx;

use crate::{
    configuration::globals::get_db_pool, price_feed::entities::TickerData,
    tokens::token_service,
};

pub async fn get_active_tickers() -> Result<Vec<TickerData>> {
    todo!();
    let pool = get_db_pool();
    // let res: Vec<TickerData> = sqlx::query!(
    //     r#"
    //     SELECT
    //         token_1,
    //         token_2,
    //         bybit_repr,
    //         chainlink_repr,
    //         pyth_repr
    //     FROM price_feed.tickers as tickers
    //     JOIN tokens.token_details as tokens on tokens.symbol = tickers.token_1
    //     WHERE tokens.is_active = true
    //     "#,
    // )
    // .fetch_all(pool)
    // .await?
    // .into_iter()
    // .map(|x| {
    //     TickerData::new(
    //         token_service::get_token_by_symbol(&x.token_1),
    //         token_service::get_token_by_symbol(&x.token_2),
    //         x.bybit_repr,
    //         x.chainlink_repr,
    //         x.pyth_repr,
    //     )
    // })
    // .collect();
    //
    // Ok(res)
}
