use compass_core::model::traversal::traversal_model_error::TraversalModelError;

#[derive(thiserror::Error, Debug)]
pub enum CompassConfigurationError {
    #[error("expected field {0} for component {1} provided by configuration")]
    ExpectedFieldForComponent(String, String),
    #[error("expected field {0} with type {1} was unable to deserialize")]
    ExpectedFieldWithType(String, String),
    #[error("unknown module {0} for component {1} provided by configuration")]
    UnknownModelNameForComponent(String, String),
    #[error(transparent)]
    SerdeDeserializationError(#[from] serde_json::Error),
    #[error(transparent)]
    TraversalModelError(#[from] TraversalModelError),
}
