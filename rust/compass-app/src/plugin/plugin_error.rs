use geo::Coord;

#[derive(thiserror::Error, Debug, Clone)]
pub enum PluginError {
    #[error("failed to parse {0} as {1}")]
    ParseError(&'static str, &'static str),
    #[error("missing field {0}")]
    MissingField(&'static str),
    #[error("error with parsing inputs: {0}")]
    InputError(&'static str),
    #[error("error with building plugin")]
    BuildError,
    #[error("nearest vertex not found for coord {0:?}")]
    NearestVertexNotFound(Coord),

}
