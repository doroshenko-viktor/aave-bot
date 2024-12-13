use alloy::providers::{ProviderBuilder, WsConnect};
use dotenv::dotenv;
use eyre::{Error, Result};
use governor::{
    clock::{QuantaClock, QuantaInstant},
    middleware::NoOpMiddleware,
    state::{InMemoryState, NotKeyed},
    Quota, RateLimiter,
};
use sqlx::postgres::PgPoolOptions;
use std::{env, num::NonZeroU32};
use tracing::debug;
use tracing_subscriber::FmtSubscriber;

use crate::{
    configuration::{AppMode, PostgresConfig, RpcProviderConfig, Settings},
    persistence::PostgresPool,
    NodeProvider,
};

pub struct Application {
    pub db_pool: PostgresPool,
    pub rpc_provider: NodeProvider,
    pub settings: Settings,
    pub rpc_rate_limiter:
        RateLimiter<NotKeyed, InMemoryState, QuantaClock, NoOpMiddleware<QuantaInstant>>,
}

pub async fn configure_app() -> Result<Application, Error> {
    dotenv().ok();
    let mode = AppMode::try_from(&env::var("APP_MODE").unwrap_or("dev".into())[..])?;
    configure_environment_defaults(&mode);
    configure_logging(&mode);
    let settings = get_settings(&mode)?;
    let db_pool = get_pool(&settings.postgres).await?;
    let rpc_provider = get_rpc_provider(&settings.rpc_config).await?;
    let rpc_rate_limiter = RateLimiter::direct(Quota::per_second(NonZeroU32::new(5).unwrap()));

    Ok(Application {
        db_pool,
        rpc_provider,
        settings,
        rpc_rate_limiter,
    })
}

fn configure_logging(mode: &AppMode) {
    let level = match mode {
        AppMode::Development => tracing::Level::DEBUG,
        AppMode::Production => tracing::Level::WARN,
        AppMode::Staging => tracing::Level::WARN,
        AppMode::Test => tracing::Level::DEBUG,
    };
    let subscriber = FmtSubscriber::builder().with_max_level(level).finish();

    tracing::subscriber::set_global_default(subscriber).expect("Unable to set global subscriber");
}

fn configure_environment_defaults(mode: &AppMode) {
    dotenv::from_filename(format!(".env.{}", mode.to_string())).ok();
    if env::var("RUST_BACKTRACE").is_err() {
        env::set_var("RUST_BACKTRACE", "1");
    }
    if env::var("RUST_LOG").is_err() {
        env::set_var("RUST_LOG", "debug");
    }
}

pub fn get_settings(mode: &AppMode) -> Result<Settings, Error> {
    let settings = Settings::new(mode)?;
    debug!("{:#?}", settings);

    Ok(settings)
}

pub async fn get_pool(config: &PostgresConfig) -> Result<PostgresPool> {
    let pool = PgPoolOptions::new()
        .max_connections(config.max_connections)
        .connect(&config.url)
        .await?;
    Ok(pool)
}

pub async fn get_rpc_provider(config: &RpcProviderConfig) -> Result<NodeProvider> {
    let ws_connect = WsConnect::new(&config.node_url);
    let provider = ProviderBuilder::new().on_ws(ws_connect).await?;

    Ok(provider)
}
