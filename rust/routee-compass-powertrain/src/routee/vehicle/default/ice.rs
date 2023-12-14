use std::sync::Arc;

use routee_compass_core::{
    model::traversal::{
        state::state_variable::StateVar, traversal_model_error::TraversalModelError,
    },
    util::unit::{
        as_f64::AsF64, Distance, DistanceUnit, Energy, EnergyUnit, Grade, GradeUnit, Speed,
        SpeedUnit,
    },
};

use crate::routee::{
    prediction::PredictionModelRecord,
    vehicle::{VehicleEnergyResult, VehicleState, VehicleType},
};

pub struct ICE {
    pub name: String,
    pub prediction_model_record: Arc<PredictionModelRecord>,
    pub max_link_energy_delta: Energy,
}

impl ICE {
    pub fn new(
        name: String,
        prediction_model_record: PredictionModelRecord,
        max_link_energy_delta: Option<Energy>,
    ) -> Self {
        let max_energy_delta = match max_link_energy_delta {
            Some(max_energy_delta) => max_energy_delta,
            None => EnergyUnit::GallonsGasoline.convert(
                Energy::new(0.25),
                prediction_model_record
                    .energy_rate_unit
                    .associated_energy_unit(),
            ),
        };
        Self {
            name,
            prediction_model_record: Arc::new(prediction_model_record),
            max_link_energy_delta: max_energy_delta,
        }
    }
}

impl VehicleType for ICE {
    fn name(&self) -> String {
        self.name.clone()
    }
    fn initial_state(&self) -> VehicleState {
        // accumulated energy
        vec![StateVar(0.0)]
    }
    fn best_case_energy(
        &self,
        distance: (Distance, DistanceUnit),
    ) -> Result<(Energy, EnergyUnit), TraversalModelError> {
        let (distance, distance_unit) = distance;
        let energy = Energy::create(
            self.prediction_model_record.ideal_energy_rate,
            self.prediction_model_record.energy_rate_unit,
            distance,
            distance_unit,
        )?;
        Ok(energy)
    }
    fn consume_energy(
        &self,
        speed: (Speed, SpeedUnit),
        grade: (Grade, GradeUnit),
        distance: (Distance, DistanceUnit),
        state: &[StateVar],
    ) -> Result<VehicleEnergyResult, TraversalModelError> {
        let (energy, energy_unit) = self
            .prediction_model_record
            .predict(speed, grade, distance)?;

        let updated_state = update_state(state, energy);

        Ok(VehicleEnergyResult {
            energy,
            energy_unit,
            updated_state,
        })
    }
    fn serialize_state(&self, state: &[StateVar]) -> serde_json::Value {
        let energy = get_energy_from_state(state);
        serde_json::json!({
            "energy": energy.as_f64(),
        })
    }

    fn serialize_state_info(&self, _state: &[StateVar]) -> serde_json::Value {
        let energy_unit = self
            .prediction_model_record
            .energy_rate_unit
            .associated_energy_unit();
        serde_json::json!({
            "energy_unit": energy_unit.to_string(),
        })
    }

    fn update_from_query(
        &self,
        _query: &serde_json::Value,
    ) -> Result<Arc<dyn VehicleType>, TraversalModelError> {
        // just return a clone of self
        Ok(Arc::new(ICE {
            name: self.name.clone(),
            prediction_model_record: self.prediction_model_record.clone(),
            max_link_energy_delta: self.max_link_energy_delta,
        }))
    }

    fn normalize_energy(&self, energy: (Energy, EnergyUnit)) -> f64 {
        let (energy, _energy_unit) = energy;
        let normalized_energy = energy / self.max_link_energy_delta;
        normalized_energy.as_f64()
    }
}

fn update_state(state: &[StateVar], energy: Energy) -> VehicleState {
    let mut new_state = Vec::with_capacity(state.len());
    new_state.push(state[0] + energy.into());
    new_state
}

fn get_energy_from_state(state: &[StateVar]) -> Energy {
    let energy = state[0].0;
    Energy::new(energy)
}
