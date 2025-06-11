use crate::model::unit::TimeUnit;

use super::turn::Turn;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uom::si::f64::Time;

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "type")]
pub struct TurnDelayModelConfig {
    table: HashMap<Turn, f64>,
    time_unit: TimeUnit,
}

pub enum TurnDelayModel {
    TabularDiscrete { table: HashMap<Turn, Time> },
}

impl From<TurnDelayModelConfig> for TurnDelayModel {
    fn from(config: TurnDelayModelConfig) -> Self {
        let table = config
            .table
            .into_iter()
            .map(|(turn, delay)| (turn, config.time_unit.to_uom(delay)))
            .collect();
        TurnDelayModel::TabularDiscrete { table }
    }
}
