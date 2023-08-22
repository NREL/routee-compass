use super::compass::{
    compass_input_field::CompassInputField,
    config::compass_configuration_error::CompassConfigurationError,
};
use crate::plugin::plugin_error::PluginError;
use compass_core::{
    algorithm::search::search_error::SearchError,
    model::traversal::traversal_model_error::TraversalModelError,
};
use config::ConfigError;

#[derive(thiserror::Error, Debug)]
pub enum AppError {
    #[error(transparent)]
    SearchError(#[from] SearchError),
    #[error(transparent)]
    TraversalModelError(#[from] TraversalModelError),
    #[error(transparent)]
    PluginError(#[from] PluginError),
    #[error(transparent)]
    IOError(#[from] std::io::Error),
    #[error(transparent)]
    CodecError(#[from] serde_json::Error),
    #[error(transparent)]
    ConfigError(#[from] ConfigError),
    #[error(transparent)]
    CompassConfigurationError(#[from] CompassConfigurationError),
    #[error("a ux component caused a failure: {0}")]
    UXError(String),
    #[error("internal error: {0}")]
    InternalError(String),
    #[error("app input JSON missing field: {0}")]
    MissingInputField(CompassInputField),
    #[error("error decoding input: {0}")]
    InvalidInput(String),
}
