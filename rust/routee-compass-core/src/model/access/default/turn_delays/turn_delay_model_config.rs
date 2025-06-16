use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::model::{access::default::turn_delays::Turn, unit::TimeUnit};

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "type")]
pub struct TurnDelayModelConfig {
    pub table: HashMap<Turn, f64>,
    pub time_unit: TimeUnit,
}
