use serde::Deserialize;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
// #[serde(rename_all = "PascalCase")]
struct ProfileToSpeedPerTimeSlot {
    profile_id: Uuid,
    speed_per_time_slot_id: Uuid,
}
