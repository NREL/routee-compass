#[derive(thiserror::Error, Debug, Clone)]
pub enum FilterModelError {
    #[error("failure building filter model: {0}")]
    BuildError(String),
    #[error("{0}")]
    FilterModelError(String),
}
