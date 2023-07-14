#[derive(thiserror::Error, Debug, Clone)]
pub enum TraversalError {
    #[error("internal error, state variable index is invalid")]
    InvalidStateVariableIndexError,
    #[error("tough stuff brah")]
    Error,
}
