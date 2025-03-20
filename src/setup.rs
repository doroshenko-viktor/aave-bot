// use alloy::providers::{self, ProviderBuilder, WsConnect, Ws};
use alloy::{
    network::{Ethereum, Network},
    providers::{
        Identity, ProviderBuilder, RootProvider,
        fillers::{BlobGasFiller, ChainIdFiller, FillProvider, GasFiller, JoinFill, NonceFiller},
    },
    transports::ws::{self, WsConnect},
};
use dotenv::dotenv;
use eyre::{Error, Result};
use governor::{Quota, RateLimiter};
use sqlx::postgres::PgPoolOptions;
use std::{env, num::NonZeroU32};
use tracing::debug;
use tracing_subscriber::FmtSubscriber;

use crate::{
    NodeWsProvider,
    configuration::{
        AppMode, PostgresConfig, RpcProviderConfig, RpcRateLimiter, Settings,
        globals::{ASSET_RULES, DB_POOL, RPC_PROVIDER, SETTINGS, get_db_pool},
    },
    shared::persistence::DbPool,
    tokens::asset_rules_aggregator::AssetRules,
};

pub struct Application {
    pub rpc_rate_limiter: RpcRateLimiter,
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

    let asset_rules = get_asset_rules(get_db_pool()).await?;
    ASSET_RULES.set(asset_rules);

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
    todo!()
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
        JoinFill<GasFiller, JoinFill<BlobGasFiller, JoinFill<NonceFiller, ChainIdFiller>>>,
    >,
    RootProvider,
>;

async fn get_rpc_provider(config: &RpcProviderConfig) -> Result<NodeWsProvider> {
    Ok(ProviderBuilder::new()
        .on_ws(WsConnect::new(&config.node_url))
        .await?)
}
