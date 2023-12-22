#[derive(thiserror::Error, Debug)]
pub enum CostError {
    #[error("failure reading CSV: {source}")]
    CsvIoError {
        #[from]
        source: csv::Error,
    },
    #[error("expected state variable name {0} not found in cost rate table")]
    StateVariableNotFound(String),
    #[error("index {0} for state variable {1} out of bounds, not found in traversal state")]
    StateIndexOutOfBounds(usize, String),
}
