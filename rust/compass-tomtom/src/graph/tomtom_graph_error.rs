#[derive(thiserror::Error, Debug)]
pub enum TomTomGraphError {
    #[error("{filename} file source was empty")]
    EmptyFileSource { filename: String },
    #[error("error in test setup")]
    IOError {
        #[from]
        source: std::io::Error,
    },
    #[error("csv read error")]
    CsvError {
        #[from]
        source: csv::Error,
    },
}
