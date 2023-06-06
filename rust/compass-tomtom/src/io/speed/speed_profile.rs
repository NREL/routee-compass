use serde::Deserialize;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
// #[serde(rename_all = "PascalCase")]
struct SpeedProfile {
    speed_profile_id: Uuid,
    monday_profile_id: Uuid,
    tuesday_profile_id: Uuid,
    wednesday_profile_id: Uuid,
    thursday_profile_id: Uuid,
    friday_profile_id: Uuid,
    saturday_profile_id: Uuid,
    sunday_profile_id: Uuid,
}
