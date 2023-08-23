#[derive(thiserror::Error, Debug, Clone)]
pub enum FrontierModelError {
    #[error("failure building frontier model")]
    BuildError,
}
