use config::{Config, ConfigError, Environment, File};
use eyre::{eyre, Error};
use serde::Deserialize;

pub enum AppMode {
    Development,
    Production,
    Staging,
    Test,
}

impl TryFrom<&str> for AppMode {
    type Error = Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "dev" => Ok(Self::Development),
            "prod" => Ok(Self::Production),
            "stage" => Ok(Self::Staging),
            "test" => Ok(Self::Test),
            _ => Err(eyre!("Couldn't parse APP_MODE from given value: {}", value)),
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
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct AppConfig {
    pub first_block: u64,
    pub indexing_step: u64,
}

#[derive(Debug, Deserialize, Clone)]
pub struct PostgresConfig {
    pub url: String,
    pub max_connections: u32,
}

#[derive(Debug, Deserialize, Clone)]
pub struct RpcProviderConfig {
    pub node_url: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Settings {
    pub config: AppConfig,
    pub rpc_config: RpcProviderConfig,
    pub postgres: PostgresConfig,
}

const CONFIG_FOLDER_PATH: &str = "bot/config/";
const DEFAULT_CONFIG_FILE_NAME: &str = "default";

impl Settings {
    pub fn new(run_mod: &AppMode) -> Result<Self, ConfigError> {
        let config_builder = Config::builder()
            .add_source(
                File::with_name(
                    format!("{}{}", CONFIG_FOLDER_PATH, DEFAULT_CONFIG_FILE_NAME).as_ref(),
                )
                .required(true),
            )
            .add_source(
                File::with_name(format!("{}{}", CONFIG_FOLDER_PATH, run_mod.to_string()).as_ref())
                    .required(false),
            )
            .add_source(Environment::with_prefix("app").separator("__"))
            .build()?;

        config_builder.try_deserialize()
    }
}
