use std::path::PathBuf;

use config::{Config, ConfigError, File};
use serde::Deserialize;

use crate::config::graph::GraphConfig;
use crate::config::plugin::PluginConfig;
use crate::config::search::SearchConfig;

const DEFAULT_FILE: &str = "compass-app/src/config/config.default.toml";

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
            .add_source(File::with_name(
                DEFAULT_FILE
            ))
            .build()?;
        s.try_deserialize()
    }

    pub fn from_file(filename: String) -> Result<Self, ConfigError> {
        let path = PathBuf::from(filename);
        let s = Config::builder()
            .add_source(File::with_name(DEFAULT_FILE))
            .add_source(File::from(path))
            .build()?;
        s.try_deserialize()
    }
}
