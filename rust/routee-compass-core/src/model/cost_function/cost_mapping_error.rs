#[derive(thiserror::Error, Debug)]
pub enum CostMappingError {
    #[error("failure reading lookup CSV: {source}")]
    LookupFileIOError {
        #[from]
        source: csv::Error,
    },
}
