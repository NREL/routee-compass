use derive_more::{Add, Mul, Sum};

/// represents time in milliseconds. can be positive or negative.
#[derive(Copy, Clone, Eq, PartialEq, Add, Mul, Sum)]
pub struct Milliseconds(pub i64);

impl Milliseconds {
    pub fn from_hours(h: f64) -> Milliseconds {
        Milliseconds((h * 3.6e+6) as i64)
    }
    pub fn from_minutes(m: f64) -> Milliseconds {
        Milliseconds((m * 60000.0) as i64)
    }
    pub fn from_seconds(s: f64) -> Milliseconds {
        Milliseconds((s * 1000.0) as i64)
    }
}
