use sqlx::query_as;
use eyre::Result;
use crate::shared::persistence::PostgresPool;

pub async fn get_liquidations_sorted_by_block(
    pool: &PostgresPool,
    from_block: i32,
    to_block: i32,
) -> Result<Vec<LiquidationEventDto>> {
    let events = query_as!(
        LiquidationEventDto,
        r#"
        select
            block_number,
            log_id,
            collateral_asset,
            debt_asset,
            "user",
            debt_to_cover,
            liquidated_collateral_amount,
            liquidator,
            receive_a_token
        from blockchain.liquidation_events
        where block_number >= $1 and block_number <= $2
        order by block_number asc;
        "#,
        from_block,
        to_block
    )
    .fetch_all(pool)
    .await?;

    Ok(events)
}
