use crate::{shared::persistence::DbPool, utils::address_ext::AddressExt};
use alloy::primitives::Address;
use eyre::Result;
use sqlx::{query_as, query_as_unchecked};

use super::AaveAccountMovementDto;

pub async fn get_movements_paginated(
    db_pool: &DbPool,
    address: Option<Address>,
    token_ids: Option<Vec<i32>>,
    event_types: Option<Vec<String>>,
    offset: u64,
    limit: u64,
) -> Result<Vec<AaveAccountMovementDto>> {
    let address = address.map(|addr| addr.to_compact_str());
    let events: Vec<AaveAccountMovementDto> = query_as_unchecked!(
        AaveAccountMovementDto,
        r#"
        select
            id,
            event_type,
            transaction_hash,
            account_address,
            token_id,
            native_token_amount,
            reserve_token_amount
        from operational.movement_events
        where
            ($3::varchar(42) is null or account_address = $3)
            and ($4::int[] is null or token_id = any($4))
            and ($5::text[] is null or event_type::text = any($5))
        order by id asc
        offset $1
        limit $2;
        "#,
        offset as i32,
        limit as i32,
        address,
        token_ids,
        event_types
    )
    .fetch_all(db_pool)
    .await?;

    Ok(events)
}
