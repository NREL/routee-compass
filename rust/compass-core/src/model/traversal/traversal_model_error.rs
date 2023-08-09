use super::state::search_state::SearchState;

#[derive(thiserror::Error, Debug, Clone)]
pub enum TraversalModelError {
    #[error("failure building traversal model from file {0}: {1}")]
    FileReadError(String, String),
    #[error("failure building traversal model")]
    BuildError,
    #[error("index {0} for {1} not found on search state {2:?}")]
    StateVectorIndexOutOfBounds(usize, String, SearchState),
    #[error("id {0} for id type {1} not found in tabular edge cost function {2}")]
    MissingIdInTabularCostFunction(String, String, String),
    #[error("tough stuff brah")]
    Error,
    #[error("prediction model failed with error {0}")]
    PredictionModel(String),
}
