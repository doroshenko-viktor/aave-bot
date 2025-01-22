use crate::shared::persistence::PostgresPool;
use eyre::Result;
use sqlx::query_as;

use super::BorrowEventDto;

pub async fn get_borrows_sorted_by_block(
    pool: &PostgresPool,
    from_block: i32,
    to_block: i32,
) -> Result<Vec<BorrowEventDto>> {
    let events = query_as!(
        BorrowEventDto,
        r#"
        select
            block_number,
            log_id,
            amount,
            borrow_rate,
            interest_rate_mode,
            on_behalf_of,
            referral_code,
            reserve,
            "user"
        from blockchain.borrow_events
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
