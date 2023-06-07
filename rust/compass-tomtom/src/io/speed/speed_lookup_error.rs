use super::{network_id::NetworkId, speed_profile_id::SpeedProfileId};

#[derive(thiserror::Error, Debug, Clone)]
pub enum SpeedLookupError {
    #[error("deserialization error")]
    DeserializationError { msg: String },
    #[error("speed profile not found: {speed_profile_id}")]
    SpeedProfileNotFound { speed_profile_id: SpeedProfileId },
    #[error("network id occurs more than once in network to speed profile table: {network_id}")]
    NonUniqueNetworkId { network_id: NetworkId },
    #[error("speed profile id occurs more than once in speed profile table: {speed_profile_id}")]
    NonUniqueSpeedProfileId { speed_profile_id: SpeedProfileId },
}
