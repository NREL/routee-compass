#[derive(thiserror::Error, Debug, Clone)]
pub enum FrontierModelError {
    #[error("failure building frontier model: {0}")]
    BuildError(String),
    #[error("edge id {0} missing from frontier model file")]
    MissingIndex(String),
}
