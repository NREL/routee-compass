use super::turn::Turn;
use crate::model::unit::Time;
use std::collections::HashMap;

pub enum TurnDelayModel {
    /// use a mapping heuristic from turn ranges to time delays
    TabularDiscrete { table: HashMap<Turn, Time> },
}
