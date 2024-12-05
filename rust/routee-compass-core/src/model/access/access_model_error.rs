use crate::model::state::state_model_error::StateModelError;

#[derive(thiserror::Error, Debug)]
pub enum AccessModelError {
    #[error("error while executing access model {name}: {error}")]
    RuntimeError { name: String, error: String },
    #[error("access model failed due to state transition error: {source}")]
    StateError {
        #[from]
        source: StateModelError,
    },
    #[error("{0}")]
    BuildError(String),
}
