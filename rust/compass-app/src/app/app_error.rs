use compass_core::algorithm::search::search_error::SearchError;

use crate::plugin::plugin_error::PluginError;

#[derive(thiserror::Error, Debug)]
pub enum AppError {
    #[error("search failure")]
    SearchError(#[from] SearchError),
    #[error("application plugin caused a failure")]
    PluginError(#[from] PluginError),
    #[error(transparent)]
    IOError(#[from] std::io::Error),
    #[error(transparent)]
    CodecError(#[from] serde_json::Error),
    #[error("a ux component caused a failure: {0}")]
    UXError(String),
    #[error("internal error: {0}")]
    InternalError(String),
}
