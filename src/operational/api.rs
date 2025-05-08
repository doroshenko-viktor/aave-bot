use crate::{
    operational::{aave_events, dto::AaveAccountMovementApiDto},
    utils::serializers::{
        deserialize_comma_separated_values_i32,
        deserialize_comma_separated_values_str, deserialize_eth_address,
    },
    web_api::{AppState, errors::ApiError},
};
use alloy::primitives::Address;
use axum::{
    Json,
    extract::{Query, State},
    response::IntoResponse,
};
use eyre::Result;
use serde::{Deserialize, Serialize};
use tracing::info;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GetAaveMovementsParams {
    #[serde(default, deserialize_with = "deserialize_eth_address")]
    pub address: Option<Address>,
    #[serde(
        default,
        deserialize_with = "deserialize_comma_separated_values_i32"
    )]
    pub tokens: Option<Vec<i32>>,
    #[serde(
        default,
        deserialize_with = "deserialize_comma_separated_values_str"
    )]
    pub event_types: Option<Vec<String>>,
    pub offset: Option<u64>,
    pub limit: Option<u64>,
}

pub async fn get_aave_movements(
    State(state): State<AppState>,
    Query(params): Query<GetAaveMovementsParams>,
) -> Result<impl IntoResponse, ApiError> {
    info!("get_aave_movements: {:?}", params);
    let movements = aave_events::get_movements_paginated(
        state.db_pool,
        params.address,
        params.tokens,
        params.event_types,
        params.offset.unwrap_or(0),
        params.limit.unwrap_or(100),
    )
    .await?
    .into_iter()
    .map(|x| x.into())
    .collect::<Vec<AaveAccountMovementApiDto>>();

    Ok(Json(movements))
}
