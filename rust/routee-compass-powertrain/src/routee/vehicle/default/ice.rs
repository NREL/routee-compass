use routee_compass_core::{
    model::traversal::{
        state::state_variable::StateVar, traversal_model_error::TraversalModelError,
    },
    model::unit::{
        as_f64::AsF64, Distance, DistanceUnit, Energy, EnergyUnit, Grade, GradeUnit, Speed,
        SpeedUnit,
    },
};
use std::sync::Arc;

use crate::routee::{
    prediction::PredictionModelRecord,
    vehicle::{VehicleEnergyResult, VehicleState, VehicleType},
};

pub struct ICE {
    pub name: String,
    pub prediction_model_record: Arc<PredictionModelRecord>,
}

impl ICE {
    pub fn new(
        name: String,
        prediction_model_record: PredictionModelRecord,
    ) -> Result<Self, TraversalModelError> {
        Ok(Self {
            name,
            prediction_model_record: Arc::new(prediction_model_record),
        })
    }
}

impl VehicleType for ICE {
    fn name(&self) -> String {
        self.name.clone()
    }
    fn state_variable_names(&self) -> Vec<String> {
        vec![String::from("energy_liquid")]
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

    fn best_case_energy_state(
        &self,
        distance: (Distance, DistanceUnit),
        state: &[StateVar],
    ) -> Result<VehicleEnergyResult, TraversalModelError> {
        let (energy, energy_unit) = self.best_case_energy(distance)?;
        let updated_state = update_state(state, energy);

        Ok(VehicleEnergyResult {
            energy,
            energy_unit,
            updated_state,
        })
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
            "energy_liquid": energy.as_f64(),
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
        }))
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
