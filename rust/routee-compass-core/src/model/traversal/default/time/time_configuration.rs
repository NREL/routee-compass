use serde::{Deserialize, Serialize};

use crate::model::unit::TimeUnit;

#[derive(Serialize, Deserialize)]
pub struct TimeConfiguration {
    /// unit to record time
    pub time_unit: TimeUnit,
}
