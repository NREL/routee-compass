use super::{map_json_key::MapJsonKey, matching_type::MatchingType};
use crate::model::{
    network::EdgeId,
    unit::{Distance, DistanceUnit, UnitError},
};

#[derive(thiserror::Error, Debug)]
pub enum MapError {
    #[error("failure building model: {0}")]
    BuildError(String),
    #[error("map geometries missing EdgeId {0}")]
    MissingEdgeId(EdgeId),
    #[error("failure matching query to map: {0}")]
    MapMatchError(String),
    #[error("this Compass instance is configured to require destinations on inputs, but the appropriate 'destination_*' fields were not found on query (looked for: {0})")]
    DestinationsRequired(MatchingType),
    #[error("cannot map match on key '{0}', must be one of [origin_x, origin_y, destination_x, destination_y]")]
    InvalidMapMatchingKey(MapJsonKey),
    #[error("input missing required field '{0}'")]
    InputMissingField(MapJsonKey),
    #[error("failure deserializing value {0} as expected type {1}")]
    InputDeserializingError(String, String),
    #[error("input has '{0}' field without required paired field '{1}'")]
    InputMissingPairedField(MapJsonKey, MapJsonKey),
    #[error("failure re-projecting geometry: {error} original: {geometry}")]
    ProjectionError { geometry: String, error: String },
    #[error("failure running map model due to numeric units: {source}")]
    UnitsFailure {
        #[from]
        source: UnitError,
    },
    #[error("result not found within distance threshold of {1}/{2}: {0}")]
    DistanceThresholdError(String, Distance, DistanceUnit),
    #[error("{0}")]
    InternalError(String),
}
