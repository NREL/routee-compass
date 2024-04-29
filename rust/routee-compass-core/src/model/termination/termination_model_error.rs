#[derive(thiserror::Error, Debug, Clone)]
pub enum TerminationModelError {
    #[error("query terminated due to {0}")]
    QueryTerminated(String),
    #[error("termination model runtime error {0}")]
    RuntimeError(String),
}
