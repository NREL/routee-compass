use serde::{Deserialize, Serialize};
use crate::model::unit::TimeUnit;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TimeTraversalConfig {
    /// time unit for state modeling
    pub time_unit: TimeUnit
}