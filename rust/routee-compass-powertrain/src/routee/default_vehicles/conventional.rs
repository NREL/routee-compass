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
    prediction_model::{PredictionModel, PredictionModelRecord},
    vehicle::{Vehicle, VehicleEnergyResult, VehicleState},
};

pub struct ConventionalVehicle {
    pub name: String,
    pub prediction_model_record: PredictionModelRecord,
}

impl ConventionalVehicle {
    pub fn new(
        name: String,
        prediction_model_record: PredictionModelRecord,
    ) -> Result<Self, TraversalModelError> {
        Ok(Self {
            name,
            prediction_model_record,
        })
    }
}

impl Vehicle for ConventionalVehicle {
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
    fn predict_energy(
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
    fn serialize_state(&self, state: &VehicleState) -> serde_json::Value {
        let energy = get_energy_from_state(state);
        serde_json::json!({
            "energy": energy.as_f64(),
        })
    }

    fn serialize_state_info(&self, _state: &VehicleState) -> serde_json::Value {
        let energy_unit = self
            .prediction_model_record
            .energy_rate_unit
            .associated_energy_unit();
        serde_json::json!({
            "energy_unit": energy_unit.to_string(),
        })
    }
}

fn update_state(state: &[StateVar], energy: Energy) -> VehicleState {
    let mut new_state = Vec::with_capacity(state.len());
    new_state.push(state[0] + energy.into());
    new_state
}

fn get_energy_from_state(state: &VehicleState) -> Energy {
    let energy = state[0].0;
    Energy::new(energy)
}
