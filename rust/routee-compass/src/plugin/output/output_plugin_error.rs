use crate::plugin::input::InputField;

#[derive(thiserror::Error, Debug)]
pub enum OutputPluginError {
    #[error("failure building input plugin: {0}")]
    BuildFailed(String),
    #[error("required query field '{0}' not found")]
    MissingExpectedQueryField(InputField),
    #[error("{0} provided without {1}")]
    MissingQueryFieldPair(InputField, InputField),
    #[error("required query field '{0}' is not of type {1}")]
    QueryFieldHasInvalidType(InputField, String),
    #[error("expected query to be a json object '{{}}' but found {0}")]
    UnexpectedQueryStructure(String),
    #[error("plugin experienced JSON error: {source}")]
    JsonError {
        #[from]
        source: serde_json::Error,
    },
    #[error("failure running plugin: {0}")]
    OutputPluginFailed(String),
    #[error("unexpected error: {0}")]
    InternalError(String),
}
