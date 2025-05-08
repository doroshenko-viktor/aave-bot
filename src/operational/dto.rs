use serde::{Deserialize, Serialize};

use crate::operational::aave_events::AaveAccountMovementDto;

#[derive(Debug, Serialize, Deserialize)]
pub struct AaveAccountMovementApiDto {
    pub id: i64,
    pub transaction_hash: String,
    pub event_type: String,
    pub account_address: String,
    pub token_id: i32,
    pub native_token_amount: String,
    pub reserve_token_amount: String,
}

impl From<AaveAccountMovementDto> for AaveAccountMovementApiDto {
    fn from(movement: AaveAccountMovementDto) -> Self {
        Self {
            id: movement.id,
            transaction_hash: movement.transaction_hash,
            event_type: movement.event_type.to_string(),
            account_address: movement.account_address,
            token_id: movement.token_id,
            native_token_amount: movement.native_token_amount.to_string(),
            reserve_token_amount: movement.reserve_token_amount.to_string(),
        }
    }
}
