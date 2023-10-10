#[derive(thiserror::Error, Debug, Clone)]
pub enum FrontierModelError {
    #[error("failure building frontier model")]
    BuildError,
    #[error("edge id {0} missing from frontier model file")]
    MissingIndex(String),
}
