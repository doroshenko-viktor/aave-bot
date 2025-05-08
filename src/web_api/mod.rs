pub mod controllers;
pub mod errors;
use axum::{Router, routing::get};
use eyre::Result;
use std::net::SocketAddr;
use tower_http::cors::{Any, CorsLayer};
use tracing::info;

use crate::{
    configuration::globals::get_db_pool, operational,
    shared::persistence::DbPool, tokens,
};

#[derive(Clone)]
pub struct AppState {
    pub db_pool: &'static DbPool,
}

pub async fn setup_web_app() -> Result<()> {
    let cors = CorsLayer::new()
        .allow_methods(Any)
        .allow_origin(Any)
        .allow_headers(Any);

    let state = AppState { db_pool: get_db_pool() };
    let app = Router::new()
        .route(
            "/movements",
            get(operational::api::get_aave_movements),
        )
        .route(
            "/tokens",
            get(tokens::api::get_availabale_tokens),
        )
        .layer(cors)
        .with_state(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    info!("Start listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app.into_make_service()).await?;

    Ok(())
}
