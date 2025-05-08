use crate::{
    shared::persistence::{
        CustomPlaceholder, DbPool, get_custom_placeholders_str,
    },
    utils::{
        address_ext::AddressExt, transaction_ext::TransactionExt,
        uint_ext::UintExt,
    },
};
use eyre::{Result, eyre};
use sqlx::{query_as, query_as_unchecked};
use tracing::info;

use super::{AaveAccountMovement, AaveAccountMovementDto};

pub async fn save_aave_account_movement(
    pool: &DbPool,
    movements: &[AaveAccountMovement],
) -> Result<()> {
    let mut query_builder = r#"INSERT INTO operational.movement_events (
            event_type,
            transaction_hash,
            account_address,
            token_id,
            native_token_amount,
            reserve_token_amount
        ) VALUES"#
        .to_string();

    query_builder.push_str(&get_custom_placeholders_str(
        vec![
            Some(CustomPlaceholder::new(
                "CAST(".to_string(),
                " AS aave_event_type)".to_string(),
            )),
            None,
            None,
            None,
            None,
            None,
        ],
        movements.len(),
    ));
    query_builder
        .push_str(r#" ON CONFLICT (transaction_hash, token_id) DO NOTHING"#);
    let mut query = sqlx::query(&query_builder);

    for movement in movements {
        query = query
            .bind(movement.event_type.to_string())
            .bind(movement.transaction_hash.to_compact_str())
            .bind(movement.account_address.to_compact_str())
            .bind(movement.token.id as i32)
            .bind(movement.native_token_amount.to_decimal())
            .bind(movement.reserve_token_amount.to_decimal());
    }

    let res = query.execute(pool).await?;

    info!("INSERTION RES: {:#?}", res);

    Ok(())
}
