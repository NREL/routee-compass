use serde::{Deserialize, Serialize};
use std::fmt::Display;

#[derive(Copy, Clone, Eq, PartialEq, Serialize, Deserialize, Debug)]
pub struct SpeedProfileId(pub u64);

impl SpeedProfileId {
    pub const UNSET: SpeedProfileId = SpeedProfileId(u64::MAX);
}

impl Display for SpeedProfileId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
