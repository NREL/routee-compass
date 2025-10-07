use crate::model::state::StateModelError;

#[derive(thiserror::Error, Debug)]
pub enum CostModelError {
    #[error("{0}")]
    BuildError(String),
    #[error("invalid cost model configuration: {0}")]
    InvalidConfiguration(String),
    #[error(
        "expected state variable name {0} not found in {1} table. possible alternatives: {{2}}"
    )]
    StateVariableNotFound(String, String, String),
    #[error("index {0} for state variable {1} out of bounds, not found in traversal state")]
    StateIndexOutOfBounds(usize, String),
    #[error("index {0} for {1} state vector is out of bounds")]
    CostVectorOutOfBounds(usize, String),
    #[error("attempting to build cost model with invalid weight names: {0:?}, should only include the following state model feature names: {1:?}")]
    InvalidWeightNames(Vec<String>, Vec<String>),
    #[error("invalid cost variables, sum of state variable coefficients must be non-zero: {0:?}")]
    InvalidCostVariables(Vec<f64>),
    #[error("failed to calculate cost due to underlying state model error: {source}")]
    StateModelError {
        #[from]
        source: StateModelError,
    },
}
