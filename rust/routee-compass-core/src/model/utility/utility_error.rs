#[derive(thiserror::Error, Debug)]
pub enum UtilityError {
    #[error("failure reading CSV: {source}")]
    CsvIoError {
        #[from]
        source: csv::Error,
    },
    #[error("expected dimension {0} not found in utility mapping")]
    StateDimensionNotFound(String),
    #[error("index {0} for dimension {1} out of bounds for traversal state")]
    StateIndexOutOfBounds(usize, String),
}
