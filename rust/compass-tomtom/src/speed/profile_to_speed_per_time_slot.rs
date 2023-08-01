use serde::Deserialize;

#[derive(Debug, Deserialize)]
// #[serde(rename_all = "PascalCase")]
struct ProfileToSpeedPerTimeSlot {
    profile_id: u64,
    speed_per_time_slot_id: u64,
}
