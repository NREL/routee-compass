use super::turn::Turn;
use crate::model::{
    traversal::access::access_model_error::AccessModelError,
    unit::{Time, TimeUnit},
};
use std::collections::HashMap;

pub enum TurnDelayModel {
    /// use a mapping heuristic from turn ranges to time delays
    TabularDiscrete {
        table: HashMap<Turn, Time>,
        time_unit: TimeUnit,
    },
}

impl TurnDelayModel {
    pub fn get_delay(
        &self,
        angle: i16,
        target_time_unit: &TimeUnit,
    ) -> Result<Time, AccessModelError> {
        match self {
            TurnDelayModel::TabularDiscrete { table, time_unit } => {
                let turn = Turn::from_angle(angle)?;
                let delay = table.get(&turn).ok_or_else(|| {
                    let name = String::from("tabular discrete turn delay model");
                    let error = format!("table missing entry for turn {}", turn.to_string());
                    AccessModelError::RuntimeError { name, error }
                })?;
                let result = time_unit.convert(*delay, target_time_unit);
                Ok(result)
            }
        }
    }
}
