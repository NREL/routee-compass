use std::{collections::HashSet, sync::Arc};

use routee_compass_core::model::{
    network::{Edge, Vertex},
    state::{InputFeature, StateFeature, StateModel, StateVariable},
    traversal::{TraversalModel, TraversalModelError},
    unit::TimeUnit,
};
use uom::{
    si::f64::{Ratio, Time},
    ConstZero,
};

use crate::model::{
    charging::charging_station_locator::{ChargingStationLocator, PowerType},
    fieldname,
};

pub struct SimpleChargingModel {
    pub charging_station_locator: Arc<ChargingStationLocator>,
    pub starting_soc: Ratio,
    pub full_soc: Ratio,
    pub charge_soc_threshold: Ratio,
    pub valid_power_types: HashSet<PowerType>,
}

impl TraversalModel for SimpleChargingModel {
    fn name(&self) -> String {
        "Simple Charging Model".to_string()
    }
    fn input_features(&self) -> Vec<InputFeature> {
        vec![
            InputFeature::Ratio {
                name: fieldname::TRIP_SOC.to_string(),
                unit: None,
            },
            InputFeature::Energy {
                name: fieldname::BATTERY_CAPACITY.to_string(),
                unit: None,
            },
        ]
    }
    fn output_features(&self) -> Vec<(String, routee_compass_core::model::state::StateFeature)> {
        vec![
            (
                fieldname::EDGE_TIME.to_string(),
                StateFeature::Time {
                    value: Time::ZERO,
                    accumulator: false,
                    output_unit: Some(TimeUnit::default()),
                },
            ),
            (
                fieldname::TRIP_TIME.to_string(),
                StateFeature::Time {
                    value: Time::ZERO,
                    accumulator: true,
                    output_unit: Some(TimeUnit::default()),
                },
            ),
            (
                fieldname::TRIP_SOC.to_string(),
                StateFeature::Ratio {
                    value: self.starting_soc,
                    accumulator: true,
                    output_unit: None,
                },
            ),
        ]
    }
    fn estimate_traversal(
        &self,
        _od: (&Vertex, &Vertex),
        _state: &mut Vec<StateVariable>,
        _state_model: &StateModel,
    ) -> Result<(), TraversalModelError> {
        Ok(())
    }
    fn traverse_edge(
        &self,
        trajectory: (&Vertex, &Edge, &Vertex),
        state: &mut Vec<StateVariable>,
        state_model: &StateModel,
    ) -> Result<(), TraversalModelError> {
        let current_soc = state_model.get_ratio(state, fieldname::TRIP_SOC)?;
        let battery_capacity = state_model.get_energy(state, fieldname::BATTERY_CAPACITY)?;
        let (_start_vertex, _edge, end_vertex) = trajectory;
        if let Some(charging_station) = self
            .charging_station_locator
            .get_station(&end_vertex.vertex_id)
        {
            let should_charge = current_soc < self.charge_soc_threshold
                && self
                    .valid_power_types
                    .contains(&charging_station.power_type);
            if should_charge {
                let soc_to_full = self.full_soc - current_soc;
                let charge_energy = soc_to_full * battery_capacity;
                let time_to_charge: Time = charge_energy / charging_station.power;

                state_model.set_ratio(state, fieldname::TRIP_SOC, &self.full_soc)?;
                state_model.add_time(state, fieldname::TRIP_TIME, &time_to_charge)?;
                state_model.add_time(state, fieldname::EDGE_TIME, &time_to_charge)?;
            }
        }
        Ok(())
    }
}
