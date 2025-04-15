// use alloy::providers::{self, ProviderBuilder, WsConnect, Ws};
use alloy::{
    providers::{
        Identity, ProviderBuilder, RootProvider,
        fillers::{
            BlobGasFiller, ChainIdFiller, FillProvider, GasFiller, JoinFill,
            NonceFiller,
        },
    },
    transports::{http::reqwest, ws::WsConnect},
};
// use dotenv::dotenv;
use dotenv::dotenv;
use eyre::{Error, Result};
use sqlx::postgres::PgPoolOptions;
use std::{env, str::FromStr};
use tracing::debug;
use tracing_subscriber::FmtSubscriber;

use crate::{
    NodeHttpProvider, NodeWsProvider,
    configuration::{
        AppMode, PostgresConfig, RpcProviderConfig, RpcRateLimiter, Settings,
        globals::{ASSET_RULES, DB_POOL, SETTINGS, get_db_pool},
    },
    shared::persistence::DbPool,
    tokens::{asset_rules_aggregator::AssetRules, token_service},
};
use chrono::Local;
use std::fmt;
use tracing_subscriber::fmt::time::FormatTime;

struct LocalTime;

impl FormatTime for LocalTime {
    fn format_time(
        &self,
        w: &mut tracing_subscriber::fmt::format::Writer<'_>,
    ) -> fmt::Result {
        write!(
            w,
            "{}",
            Local::now().format("%Y-%m-%d %H:%M:%S")
        )
    }
}

pub struct Application {
    pub rpc_rate_limiter: RpcRateLimiter,
}

/**
 * Configuration relies on some global variables, so considering downsides need to check initialize order to avoid failures
 */
pub async fn configure_app() -> eyre::Result<()> {
    dotenv().ok();
    let mode =
        AppMode::try_from(&env::var("APP_MODE").unwrap_or("dev".into())[..])?;
    configure_environment_defaults(&mode);
    configure_logging(&mode);

    let settings = get_settings(&mode)?;
    SETTINGS.set(settings)?;
    let Settings { postgres, rpc_config, .. } = SETTINGS.get().unwrap().clone();

    let db_pool = get_pool(&postgres).await?;
    token_service::setup_tokens(&db_pool).await?;

    DB_POOL.set(db_pool)?;

    let asset_rules = get_asset_rules(get_db_pool()).await?;
    ASSET_RULES.set(asset_rules)?;

    Ok(())
}

fn configure_logging(mode: &AppMode) {
    let level = match mode {
        AppMode::Development => tracing::Level::DEBUG,
        AppMode::Production => tracing::Level::WARN,
        AppMode::Staging => tracing::Level::INFO,
        AppMode::Test => tracing::Level::DEBUG,
    };
    let subscriber = FmtSubscriber::builder()
        .with_max_level(level)
        .with_timer(LocalTime)
        .finish();

    tracing::subscriber::set_global_default(subscriber)
        .expect("Unable to set global subscriber");
}

fn configure_environment_defaults(mode: &AppMode) {
    dotenv::from_filename(format!(".env.{}", mode.to_string())).ok();
    unsafe {
        if env::var("RUST_BACKTRACE").is_err() {
            env::set_var("RUST_BACKTRACE", "1");
        }
        if env::var("RUST_LOG").is_err() {
            env::set_var("RUST_LOG", "debug");
        }
    }
}

fn get_settings(mode: &AppMode) -> Result<Settings, Error> {
    let settings = Settings::new(mode)?;
    debug!("{:#?}", settings);

    Ok(settings)
}

async fn get_asset_rules(pool: &DbPool) -> Result<AssetRules> {
    // TODO: add tables with asset rules and read them on startup
    Ok(AssetRules {})
}

async fn get_pool(config: &PostgresConfig) -> Result<DbPool> {
    let pool = PgPoolOptions::new()
        .max_connections(config.max_connections)
        .connect(&config.url)
        .await?;
    Ok(pool)
}

fn get_pool_lazy(config: &PostgresConfig) -> Result<DbPool> {
    Ok(PgPoolOptions::new()
        .max_connections(config.max_connections)
        .connect_lazy(&config.url)?)
}

pub type WsNodeProvider = FillProvider<
    JoinFill<
        Identity,
        JoinFill<
            GasFiller,
            JoinFill<BlobGasFiller, JoinFill<NonceFiller, ChainIdFiller>>,
        >,
    >,
    RootProvider,
>;

pub async fn get_new_stream_rpc_provider(
    config: &RpcProviderConfig,
) -> Result<NodeWsProvider> {
    let provider = ProviderBuilder::new()
        .on_ws(WsConnect::new(&config.node_ws_url))
        .await?;

    Ok(provider)
}

pub fn get_new_rpc_provider(
    config: &RpcProviderConfig,
) -> Result<NodeHttpProvider> {
    let provider = ProviderBuilder::new().on_http(reqwest::Url::from_str(
        &config.node_url,
    )?);

    Ok(provider)
}
