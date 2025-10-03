use super::{Turn, TurnDelayModelConfig};
use std::collections::HashMap;
use uom::si::f64::Time;

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
