use crate::{
    tokens::{dto::TokenDto, persistence},
    web_api::{AppState, errors::ApiError},
};
use axum::{Json, extract::State, response::IntoResponse};
use eyre::Result;
use tracing::info;

pub async fn get_availabale_tokens(
    State(state): State<AppState>,
) -> Result<impl IntoResponse, ApiError> {
    info!("get_availabale_tokens");
    let tokens = persistence::get_tokens(state.db_pool)
        .await?
        .map(|x| x.into())
        .collect::<Vec<TokenDto>>();

    Ok(Json(tokens))
}
