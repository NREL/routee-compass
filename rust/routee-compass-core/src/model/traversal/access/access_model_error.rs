#[derive(thiserror::Error, Debug)]
pub enum AccessModelError {
    #[error("error while executing access model {name}: {error}")]
    RuntimeError { name: String, error: String },
}
