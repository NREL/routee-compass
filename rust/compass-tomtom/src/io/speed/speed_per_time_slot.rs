use serde::Deserialize;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
// #[serde(rename_all = "PascalCase")]
struct SpeedPerTimeSlot {
    speed_per_time_slot_id: Uuid,
    relative_speed: u16,
    time_slot: u16,
}
