#[derive(thiserror::Error, Debug)]
pub enum AccessModelError {
    #[error("error while executing access model {name}: {error}")]
    RuntimeError { name: String, error: String },
    #[error(transparent)]
    SerdeJsonError(#[from] serde_json::Error),
}
