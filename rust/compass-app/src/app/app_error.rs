use compass_core::algorithm::search::search_error::SearchError;

use crate::plugin::plugin_error::PluginError;

#[derive(thiserror::Error, Debug)]
pub enum AppError {
    #[error("search failure")]
    SearchError(#[from] SearchError),
    #[error("application plugin caused a failure")]
    PluginError(#[from] PluginError),
    #[error("internal error: {0}")]
    InternalError(String),
}
