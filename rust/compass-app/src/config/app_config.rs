use std::path::PathBuf;

use config::{Config, ConfigError, File, FileFormat};
use serde::Deserialize;

use crate::config::graph::GraphConfig;
use crate::config::plugin::PluginConfig;
use crate::config::search::SearchConfig;

const DEFAULT_FILE: &str = include_str!("config.default.toml");

#[derive(Debug, Deserialize)]
pub struct AppConfig {
    pub graph: GraphConfig,
    pub search: SearchConfig,
    pub plugin: PluginConfig,

    pub query_timeout_ms: u64,
}

impl AppConfig {
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
