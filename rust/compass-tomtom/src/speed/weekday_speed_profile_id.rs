use serde::{Deserialize, Serialize};
use std::fmt::Display;

#[derive(Copy, Clone, Eq, PartialEq, Serialize, Deserialize, Debug)]
pub struct WeekdaySpeedProfileId(pub u64);

impl WeekdaySpeedProfileId {
    pub const UNSET: WeekdaySpeedProfileId = WeekdaySpeedProfileId(u64::MAX);
}

impl Display for WeekdaySpeedProfileId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
