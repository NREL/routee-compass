use super::compass_app_args::CompassAppArgs;
use super::graph::GraphConfig;
use super::plugin::PluginConfig;
use super::search::SearchConfig;
use config::{Config, ConfigError, File, FileFormat};
use serde::Deserialize;
use std::path::PathBuf;

const DEFAULT_FILE: &str = include_str!("config.default.toml");

#[derive(Debug, Deserialize)]
pub struct CompassAppConfig {
    pub graph: GraphConfig,
    pub search: SearchConfig,
    pub plugin: PluginConfig,

    pub query_timeout_ms: u64,
}

impl TryFrom<&CompassAppArgs> for CompassAppConfig {
    type Error = ConfigError;

    /// build
    fn try_from(value: &CompassAppArgs) -> Result<Self, Self::Error> {
        match &value.config {
            Some(config_file) => {
                let config = CompassAppConfig::from_path(&config_file)?;
                log::debug!("Using config file: {:?}", config_file);
                Ok(config)
            }
            None => {
                let config = CompassAppConfig::default()?;
                log::debug!("Using default config");
                Ok(config)
            }
        }
    }
}

impl CompassAppConfig {
    pub fn default() -> Result<Self, ConfigError> {
        let s = Config::builder()
            .add_source(File::from_str(DEFAULT_FILE, FileFormat::Toml))
            .build()?;
        s.try_deserialize()
    }

    pub fn from_path(path: &PathBuf) -> Result<Self, ConfigError> {
        let s = Config::builder()
            .add_source(File::from_str(DEFAULT_FILE, FileFormat::Toml))
            .add_source(File::from(path.clone()))
            .build()?;
        s.try_deserialize()
    }
}
