use crate::model::unit::TimeUnit;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct TimeConfiguration {
    /// unit to record time
    pub time_unit: TimeUnit,
}
