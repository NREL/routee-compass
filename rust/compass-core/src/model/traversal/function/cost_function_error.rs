use crate::model::traversal::state::search_state::StateVector;

#[derive(thiserror::Error, Debug, Clone)]
pub enum CostFunctionError {
    #[error("index {0} for {1} not found on state vector {2:?}")]
    StateVectorIndexOutOfBounds(usize, String, StateVector),
    #[error("failure reading source file {0}: {1}")]
    FileReadError(String, String),
}
