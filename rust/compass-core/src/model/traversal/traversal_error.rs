#[derive(thiserror::Error, Debug, Clone)]
pub enum TraversalError {
    #[error("tough stuff brah")]
    Error,
}
