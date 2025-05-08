pub mod globals;
pub mod persistent_config;
use config::{Config, ConfigError, Environment, File};
use eyre::eyre;
use eyre::{Error, Ok, Result};
pub use persistent_config::{config_get, config_set, get_as};
use serde::Deserialize;

use governor::{
    RateLimiter,
    clock::{QuantaClock, QuantaInstant},
    middleware::NoOpMiddleware,
    state::{InMemoryState, NotKeyed},
};

use crate::app::config::AppConfig;
use crate::shared::config::BlockchainConfig;

pub type RpcRateLimiter = RateLimiter<
    NotKeyed,
    InMemoryState,
    QuantaClock,
    NoOpMiddleware<QuantaInstant>,
>;

pub enum AppMode {
    Development,
    Production,
    Staging,
    Test,
    Debug,
}

impl TryFrom<&str> for AppMode {
    type Error = Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "dev" => Ok(Self::Development),
            "prod" => Ok(Self::Production),
            "stage" => Ok(Self::Staging),
            "test" => Ok(Self::Test),
            "debug" => Ok(Self::Debug),
            _ => Err(eyre!(
                "Couldn't parse APP_MODE from given value: {}",
                value
            )),
        }
    }
}

impl ToString for AppMode {
    fn to_string(&self) -> String {
        match self {
            AppMode::Development => "dev".into(),
            AppMode::Production => "prod".into(),
            AppMode::Staging => "stage".into(),
            AppMode::Test => "test".into(),
            AppMode::Debug => "debug".into(),
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct PostgresConfig {
    pub url: String,
    pub max_connections: u32,
}

#[derive(Debug, Deserialize, Clone)]
pub struct RpcProviderConfig {
    pub max_node_credits: u32,
    pub max_node_burst_size: u32,
    pub max_blockchain_retry_count: u8,
    pub node_url: String,
    pub node_ws_url: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Settings {
    pub app: AppConfig,
    pub rpc_config: RpcProviderConfig,
    pub postgres: PostgresConfig,
    pub blockchain: BlockchainConfig,
}

const CONFIG_FOLDER_PATH: &str = "./config/";
const DEFAULT_CONFIG_FILE_NAME: &str = "default";

impl Settings {
    pub fn new(run_mod: &AppMode) -> Result<Self, ConfigError> {
        let config_builder = Config::builder()
            .add_source(
                File::with_name(
                    format!(
                        "{}{}",
                        CONFIG_FOLDER_PATH, DEFAULT_CONFIG_FILE_NAME
                    )
                    .as_ref(),
                )
                .required(true),
            )
            .add_source(
                File::with_name(
                    format!(
                        "{}{}",
                        CONFIG_FOLDER_PATH,
                        run_mod.to_string()
                    )
                    .as_ref(),
                )
                .required(false),
            )
            .add_source(Environment::with_prefix("app").separator("__"))
            .build()?;

        config_builder.try_deserialize()
    }
}
