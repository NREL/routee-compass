use super::map_json_key::MapJsonKey;

#[derive(thiserror::Error, Debug)]
pub enum MapError {
    #[error("failure matching query to map: {0}")]
    MapMatchError(String),
    #[error("cannot map match on key '{0}', must be one of [origin_x, origin_y, destination_x, destination_y]")]
    InvalidMapMatchingKey(MapJsonKey),
    #[error("input missing required field '{0}'")]
    InputMissingField(MapJsonKey),
    #[error("failure deserializing value {0} as expected type {1}")]
    InputDeserializingError(String, String),
    #[error("input has '{0}' field without required paired field '{1}'")]
    InputMissingPairedField(MapJsonKey, MapJsonKey),
}
