#[derive(thiserror::Error, Debug, Clone)]
pub enum FrontierModelError {
    #[error("failure building frontier model: {0}")]
    BuildError(String),
    #[error("{0}")]
    FrontierModelError(String),
}
