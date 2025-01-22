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
use std::{env, num::NonZeroU32, sync::Arc};
use tracing::debug;
use tracing_subscriber::FmtSubscriber;

use crate::{
    configuration::{
        globals::{DB_POOL, RPC_PROVIDER, SETTINGS},
        AppMode, PostgresConfig, RpcProviderConfig, Settings,
    },
    shared::persistence::PostgresPool,
    NodeProvider,
};

pub struct Application {
    pub rpc_rate_limiter:
        RateLimiter<NotKeyed, InMemoryState, QuantaClock, NoOpMiddleware<QuantaInstant>>,
}

/**
 * Configuration relies on some global variables, so considering downsides need to check initialize order to avoid failures
 */
pub async fn configure_app() -> Result<Application, Error> {
    dotenv().ok();
    let mode = AppMode::try_from(&env::var("APP_MODE").unwrap_or("dev".into())[..])?;
    configure_environment_defaults(&mode);
    configure_logging(&mode);

    let settings = get_settings(&mode)?;
    SETTINGS.set(settings);
    let Settings {
        postgres,
        rpc_config,
        ..
    } = SETTINGS.get().unwrap().clone();

    let db_pool = get_pool(&postgres).await?;
    DB_POOL.set(db_pool);

    let rpc_provider = get_rpc_provider(&rpc_config).await?;
    RPC_PROVIDER.set(rpc_provider);

    let rpc_rate_limiter = RateLimiter::direct(Quota::per_second(NonZeroU32::new(5).unwrap()));

    Ok(Application { rpc_rate_limiter })
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

pub fn get_pool_lazy(config: &PostgresConfig) -> Result<PostgresPool> {
    Ok(PgPoolOptions::new()
        .max_connections(config.max_connections)
        .connect_lazy(&config.url)?)
}

pub async fn get_rpc_provider(config: &RpcProviderConfig) -> Result<NodeProvider> {
    let ws_connect = WsConnect::new(&config.node_url);
    let provider = ProviderBuilder::new().on_ws(ws_connect).await?;

    Ok(provider)
}
