use compass_core::model::graph::vertex_id::VertexId;

#[derive(thiserror::Error, Debug)]
pub enum TomTomGraphError {
    #[error("{filename} file source was empty")]
    EmptyFileSource { filename: String },
    #[error("error in test setup")]
    IOError {
        #[from]
        source: std::io::Error,
    },
    #[error("csv error: {source}")]
    CsvError {
        #[from]
        source: csv::Error,
    },
    #[error("adjacency list missing vertex {0}")]
    AdjacencyVertexMissing(VertexId),
}
