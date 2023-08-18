use compass_core::{
    algorithm::search::search_error::SearchError,
    model::traversal::traversal_model_error::TraversalModelError,
};

use crate::plugin::plugin_error::PluginError;

use super::compass::compass_input_field::CompassInputField;

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
    #[error("a ux component caused a failure: {0}")]
    UXError(String),
    #[error("internal error: {0}")]
    InternalError(String),
    #[error("app input JSON missing field: {0}")]
    MissingInputField(CompassInputField),
    #[error("error decoding input: {0}")]
    InvalidInput(String),
}
