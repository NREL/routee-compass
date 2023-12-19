#[derive(thiserror::Error, Debug)]
pub enum UtilityError {
    #[error("failure reading CSV: {source}")]
    CsvIoError {
        #[from]
        source: csv::Error,
    },
}
