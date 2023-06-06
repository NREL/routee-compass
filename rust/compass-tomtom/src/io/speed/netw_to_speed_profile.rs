use serde::Deserialize;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
// #[serde(rename_all = "PascalCase")]
struct Netw2SpeedProfile {
    netw_id: Uuid,
    speed_profile_id: Uuid,
    validity_direction: i8,
}
