#[derive(thiserror::Error, Debug)]
pub enum CacheError {
    #[error("Could not build cache policy due to {0}")]
    BuildError(String),
    #[error("Could not get value from cache due to {0}")]
    RuntimeError(String),
}
