use super::WithdrawEventDto;
use crate::shared::persistence::PostgresPool;
use eyre::Result;
use sqlx::query_as;

pub async fn get_withdraw_events(
    pool: &PostgresPool,
    from_block: i32,
    to_block: i32,
) -> Result<Vec<WithdrawEventDto>> {
    let events = query_as!(
        WithdrawEventDto,
        r#"
            select
                block_number,
                log_id,
                address,
                reserve,
                "user",
                "to",
                amount
            from blockchain.withdraw_events
            where block_number >= $1 and block_number <= $2
        "#,
        from_block,
        to_block
    )
    .fetch_all(pool)
    .await?;

    Ok(events)
}
