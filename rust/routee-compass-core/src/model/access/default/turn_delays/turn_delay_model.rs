use super::turn::Turn;
use crate::model::unit::{Time, TimeUnit};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum TurnDelayModel {
    /// use a mapping heuristic from turn ranges to time delays
    TabularDiscrete {
        table: HashMap<Turn, Time>,
        time_unit: TimeUnit,
    },
    // /// use a mapping heuristic from turn ranges and road class transitions
    // /// to time delays
    // /// TODO:
    // ///   - first, move ConfigJsonExtension to core crate
    // ///   - then we can write a TurnDelayModel::new(&serde_json::Value) method which can
    // ///     use the JSON extension methods
    // ///
    // TabularDiscreteWithRoadClasses {
    //     table: HashMap<(Turn, u8, u8), Time>,
    //     road_classes: Box<[u8]>,
    //     time_unit: TimeUnit,
    // },
}
