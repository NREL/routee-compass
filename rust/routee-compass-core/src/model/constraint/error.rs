use crate::model::state::StateModelError;

#[derive(thiserror::Error, Debug, Clone)]
pub enum ConstraintModelError {
    #[error("failure building constraint model: {0}")]
    BuildError(String),
    #[error("{0}")]
    ConstraintModelError(String),
    #[error("Failure with state model: {source}")]
    StateModelError {
        #[from]
        source: StateModelError,
    },
}
