use crate::model::unit::TimeUnit;
use std::collections::HashMap;

use super::Turn;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "type")]
pub struct TurnDelayModelConfig {
    pub table: HashMap<Turn, f64>,
    pub time_unit: TimeUnit,
}
