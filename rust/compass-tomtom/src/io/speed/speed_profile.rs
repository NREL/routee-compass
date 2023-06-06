use serde::Deserialize;

#[derive(Debug, Deserialize)]
// #[serde(rename_all = "PascalCase")]
struct SpeedProfile {
    speed_profile_id: u64,
    monday_profile_id: u64,
    tuesday_profile_id: u64,
    wednesday_profile_id: u64,
    thursday_profile_id: u64,
    friday_profile_id: u64,
    saturday_profile_id: u64,
    sunday_profile_id: u64,
}
