use crate::model::state::StateModelError;

#[derive(thiserror::Error, Debug, Clone)]
pub enum LabelModelError {
    #[error("{0}")]
    LabelModelError(String),
    #[error("failure building label due to search state: {source}")]
    StateError {
        #[from]
        source: StateModelError,
    },
}
