#[derive(thiserror::Error, Debug, Clone)]
pub enum UnitError {
    #[error("unable to parse {0} as a number")]
    NumericParsingError(String),
    #[error("failure due to numeric precision: {0}")]
    PrecisionError(String),
    #[error("zero division error: {0}")]
    ZeroDivisionError(String),
}
