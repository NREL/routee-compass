use compass_core::algorithm::search::search_error::SearchError;

#[derive(thiserror::Error, Debug, Clone)]
pub enum AppError {
    #[error("search failure")]
    SearchError(#[from] SearchError),
    #[error("internal error: {0}")]
    InternalError(String),
}
