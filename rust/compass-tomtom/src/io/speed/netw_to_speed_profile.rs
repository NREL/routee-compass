use serde::Deserialize;

#[derive(Debug, Deserialize)]
// #[serde(rename_all = "PascalCase")]
struct Netw2SpeedProfile {
    netw_id: u64,
    speed_profile_id: u64,
    validity_direction: i8,
}
