use derive_more::{Add, Mul, Sum};

use super::milliseconds::Milliseconds;

/// represents time in seconds. can be positive or negative.
#[derive(Copy, Clone, Eq, PartialEq, Add, Mul, Sum)]
pub struct Seconds(pub i64);

impl Seconds {
    pub fn from_hours(h: f64) -> Seconds {
        Seconds((h * 3600) as i64)
    }
    pub fn from_minutes(m: f64) -> Seconds {
        Seconds((m * 60) as i64)
    }
    pub fn from_seconds(s: f64) -> Seconds {
        Seconds((s * 1000.0) as i64)
    }
    pub fn to_milliseconds(&self) -> Milliseconds {
        Milliseconds(self.0 * 1000)
    }
}
