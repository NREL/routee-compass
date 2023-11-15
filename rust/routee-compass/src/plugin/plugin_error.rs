use std::path::PathBuf;

use routee_compass_core::{
    algorithm::search::search_error::SearchError, model::road_network::edge_id::EdgeId,
};

#[derive(thiserror::Error, Debug)]
pub enum PluginError {
    #[error("failed to parse {0} as {1}")]
    ParseError(String, String),
    #[error("missing field {0}")]
    MissingField(String),
    #[error("error with parsing inputs: {0}")]
    InputError(String),
    #[error("error with building plugin")]
    BuildError,
    #[error("{0}")]
    PluginFailed(String),
    #[error("unable to read file {0} due to {1}")]
    FileReadError(PathBuf, String),
    #[error(transparent)]
    JsonError(#[from] serde_json::Error),
    #[error(transparent)]
    GeoJsonError(#[from] geojson::Error),
    #[error(transparent)]
    CsvReadError(#[from] csv::Error),
    #[error("geometry missing for edge id {0}")]
    EdgeGeometryMissing(EdgeId),
    #[error("uuid missing for edge id {0}")]
    UUIDMissing(usize),
    #[error(transparent)]
    SearchError(#[from] SearchError),
    #[error("expected query to be a json object '{{}}' but found {0}")]
    UnexpectedQueryStructure(String),
    #[error("unexpected error {0}")]
    InternalError(String),
}
