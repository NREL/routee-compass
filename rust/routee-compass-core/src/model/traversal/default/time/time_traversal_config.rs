use crate::model::unit::TimeUnit;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TimeTraversalConfig {
    /// time unit for state modeling
    pub time_unit: TimeUnit,
}
