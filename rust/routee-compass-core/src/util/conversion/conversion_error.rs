use std::num::ParseIntError;

#[derive(thiserror::Error, Debug)]
pub enum ConversionError {
    #[error("could not decode {0} as {1}")]
    DecoderError(String, String),
    #[error(transparent)]
    SerdeJsonError {
        #[from]
        source: serde_json::Error,
    },
    #[error(transparent)]
    RegexError(#[from] regex::Error),
    #[error(transparent)]
    ParseIntError(#[from] ParseIntError),
}
