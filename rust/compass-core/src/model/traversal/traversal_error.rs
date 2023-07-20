#[derive(thiserror::Error, Debug, Clone)]
pub enum TraversalError {
    #[error("internal error, state variable index is invalid")]
    InvalidStateVariableIndexError,
    #[error("id {0} for id type {1} not found in tabular edge cost function {2}")]
    MissingIdInTabularCostFunction(String, String, String),
    #[error("tough stuff brah")]
    Error,
}
