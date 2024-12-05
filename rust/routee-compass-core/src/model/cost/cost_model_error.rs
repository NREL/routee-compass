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
    #[error("invalid cost variables, sum of state variable coefficients must be non-zero")]
    InvalidCostVariables,
}
