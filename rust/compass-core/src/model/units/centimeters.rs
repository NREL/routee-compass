use serde::Deserialize;

use super::{cm_per_second::CmPerSecond, milliseconds::Milliseconds};

#[derive(Copy, Clone, Eq, PartialEq, Deserialize, Debug, Default)]
pub struct Centimeters(pub i64);

impl Centimeters {
    /// computes the travel time for traversing this distance of centimeters at
    /// the given speed. produces time in seconds
    pub fn travel_time_seconds(&self, speed: &CmPerSecond) -> f64 {
        self.0 as f64 / speed.0 as f64
    }

    /// computes the travel time for traversing this distance of centimeters at
    /// the given speed. produces time in milliseconds
    pub fn travel_time_millis(&self, speed: &CmPerSecond) -> Milliseconds {
        let t = self.travel_time_seconds(speed) * 1000.0;
        Milliseconds(t as i64)
    }
}
