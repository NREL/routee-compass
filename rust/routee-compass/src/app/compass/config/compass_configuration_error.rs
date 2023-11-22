use config::ConfigError;
use routee_compass_core::{
    model::{
        road_network::graph_error::GraphError,
        traversal::traversal_model_error::TraversalModelError,
    },
    util::conversion::conversion_error::ConversionError,
};

use crate::plugin::plugin_error::PluginError;

#[derive(thiserror::Error, Debug)]
pub enum CompassConfigurationError {
    #[error("expected field {0} for component {1} provided by configuration")]
    ExpectedFieldForComponent(String, String),
    #[error("expected field {0} with type {1} was unable to deserialize")]
    ExpectedFieldWithType(String, String),
    #[error("expected field {0} for component {1} had unrecognized value {2}")]
    ExpectedFieldWithTypeUnrecognized(String, String, String),
    #[error("unknown module {0} for component {1} provided by configuration")]
    UnknownModelNameForComponent(String, String),
    #[error("test {0} test {1} test {2}")]
    FileNotFoundForComponent(String, String, String),
    #[error("could not normalize incoming file {0}")]
    FileNormalizationError(String),
    #[error("Could not find incoming configuration file, tried {0} and {1}. Make sure the file exists and that the config key ends with '_input_file'")]
    FileNormalizationNotFound(String, String),
    #[error("{0}")]
    InsertError(String),
    #[error(transparent)]
    GraphError(#[from] GraphError),
    #[error(transparent)]
    ConfigError(#[from] ConfigError),
    #[error(transparent)]
    IoError(#[from] std::io::Error),
    #[error(transparent)]
    SerdeDeserializationError(#[from] serde_json::Error),
    #[error(transparent)]
    ConversionError(#[from] ConversionError),
    #[error(transparent)]
    TraversalModelError(#[from] TraversalModelError),
    #[error(transparent)]
    PluginError(#[from] PluginError),
}
