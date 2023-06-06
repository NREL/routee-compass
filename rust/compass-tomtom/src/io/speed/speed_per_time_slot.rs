use serde::Deserialize;

#[derive(Debug, Deserialize)]
// #[serde(rename_all = "PascalCase")]
struct SpeedPerTimeSlot {
    speed_per_time_slot_id: u64,
    relative_speed: u16,
    time_slot: u16,
}
