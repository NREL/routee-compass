use super::input::InputPluginError;
use super::output::OutputPluginError;

#[derive(thiserror::Error, Debug)]
pub enum PluginError {
    #[error("failure building plugin: {0}")]
    BuildFailed(String),
    #[error("required query field '{0}' for plugin {1} not found")]
    MissingExpectedQueryField(String, String),
    #[error("failure running input plugin: {source}")]
    InputPluginFailed {
        #[from]
        source: InputPluginError,
    },
    #[error("failure running output plugin: {source}")]
    OutputPluginFailed {
        #[from]
        source: OutputPluginError,
    },
    #[error("plugin experienced JSON error: {source}")]
    JsonError {
        #[from]
        source: serde_json::Error,
    },
    #[error("expected query to be a json object '{{}}' but found {0}")]
    UnexpectedQueryStructure(String),
    #[error("unexpected error: {0}")]
    InternalError(String),
}
