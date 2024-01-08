use routee_compass_core::{
    model::traversal::{
        state::{state_variable::StateVar, traversal_state::TraversalState},
        traversal_model_error::TraversalModelError,
    },
    model::unit::{
        as_f64::AsF64, Distance, DistanceUnit, Energy, EnergyUnit, Grade, GradeUnit, Speed,
        SpeedUnit,
    },
};
use std::sync::Arc;

use crate::routee::{
    prediction::PredictionModelRecord,
    vehicle::{vehicle_type::VehicleType, VehicleEnergyResult},
};

pub struct BEV {
    pub name: String,
    pub prediction_model_record: Arc<PredictionModelRecord>,
    pub battery_capacity: Energy,
    pub starting_battery_energy: Energy,
    pub battery_energy_unit: EnergyUnit,
}

impl BEV {
    pub fn new(
        name: String,
        prediction_model_record: PredictionModelRecord,
        battery_capacity: Energy,
        starting_battery_energy: Energy,
        battery_energy_unit: EnergyUnit,
    ) -> Self {
        Self {
            name,
            prediction_model_record: Arc::new(prediction_model_record),
            battery_capacity,
            starting_battery_energy,
            battery_energy_unit,
        }
    }
}

impl VehicleType for BEV {
    fn name(&self) -> String {
        self.name.clone()
    }
    fn state_variable_names(&self) -> Vec<String> {
        vec![
            String::from("energy_electric"),
            String::from("battery_state"),
        ]
    }
    fn initial_state(&self) -> TraversalState {
        vec![
            StateVar(0.0),                                   // accumulated electrical energy
            StateVar(self.starting_battery_energy.as_f64()), // battery energy remaining
        ]
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
        let (electrical_energy, electrical_energy_unit) = self.best_case_energy(distance)?;
        let updated_state = update_state(state, electrical_energy, self.battery_capacity);

        Ok(VehicleEnergyResult {
            energy: electrical_energy,
            energy_unit: electrical_energy_unit,
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
        let (electrical_energy, electrical_energy_unit) = self
            .prediction_model_record
            .predict(speed, grade, distance)?;

        let updated_state = update_state(state, electrical_energy, self.battery_capacity);

        Ok(VehicleEnergyResult {
            energy: electrical_energy,
            energy_unit: electrical_energy_unit,
            updated_state,
        })
    }
    fn serialize_state(&self, state: &[StateVar]) -> serde_json::Value {
        let energy_electric = get_electrical_energy_from_state(state);
        let battery_soc_percent = get_battery_soc_percent(self, state);
        serde_json::json!({
            "energy_electric": energy_electric.as_f64(),
            "battery_soc_percent": battery_soc_percent,
        })
    }

    fn serialize_state_info(&self, _state: &[StateVar]) -> serde_json::Value {
        let battery_energy_unit = self.battery_energy_unit;
        serde_json::json!({
            "energy_unit": battery_energy_unit.to_string(),
        })
    }

    fn update_from_query(
        &self,
        query: &serde_json::Value,
    ) -> Result<Arc<dyn VehicleType>, TraversalModelError> {
        let starting_soc_percent = match query.get("starting_soc_percent".to_string()) {
            Some(soc_string) => soc_string.as_f64().ok_or_else(|| {
                TraversalModelError::BuildError(
                    "Expected 'starting_soc_percent' value to be numeric".to_string(),
                )
            })?,
            None => 100.0,
        };
        if !(0.0..=100.0).contains(&starting_soc_percent) {
            return Err(TraversalModelError::BuildError(
                "Expected 'starting_soc_percent' value to be between 0 and 100".to_string(),
            ));
        }
        let starting_battery_energy = self.battery_capacity * (starting_soc_percent / 100.0);

        let new_bev = BEV {
            name: self.name.clone(),
            prediction_model_record: self.prediction_model_record.clone(),
            battery_capacity: self.battery_capacity,
            starting_battery_energy,
            battery_energy_unit: self.battery_energy_unit,
        };

        Ok(Arc::new(new_bev))
    }
}

fn update_state(
    state: &[StateVar],
    electrical_energy: Energy,
    battery_energy_capacity: Energy,
) -> TraversalState {
    let mut updated_state = Vec::with_capacity(state.len());

    // accumulated electrical energy
    updated_state.push(state[0] + electrical_energy.into());

    // remaining battery energy
    let current_battery_energy = get_remaining_battery_energy_from_state(state);

    // don't let the battery energy go below 0 or above the battery capacity
    let new_battery_energy = (current_battery_energy - electrical_energy)
        .max(Energy::new(0.0))
        .min(battery_energy_capacity);

    updated_state.push(new_battery_energy.into());

    updated_state
}

fn get_electrical_energy_from_state(state: &[StateVar]) -> Energy {
    Energy::new(state[0].0)
}

fn get_remaining_battery_energy_from_state(state: &[StateVar]) -> Energy {
    Energy::new(state[1].0)
}

fn get_battery_soc_percent(vehicle: &BEV, state: &[StateVar]) -> f64 {
    let battery_energy_unit = vehicle.battery_energy_unit;

    let battery_capacity_kwh =
        battery_energy_unit.convert(vehicle.battery_capacity, EnergyUnit::KilowattHours);

    let remaining_battery_energy = get_remaining_battery_energy_from_state(state);

    let remaining_battery_energy_kwh =
        battery_energy_unit.convert(remaining_battery_energy, EnergyUnit::KilowattHours);

    (remaining_battery_energy_kwh.as_f64() / battery_capacity_kwh.as_f64()) * 100.0
}

#[cfg(test)]
mod tests {
    use routee_compass_core::model::unit::{EnergyRate, EnergyRateUnit};

    use crate::routee::{prediction::load_prediction_model, prediction::model_type::ModelType};

    use super::*;

    use std::path::PathBuf;

    fn mock_vehicle(starting_soc_percent: f64) -> BEV {
        let model_file_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join("routee")
            .join("test")
            .join("2017_CHEVROLET_Bolt.bin");

        let model_record = load_prediction_model(
            "Chevy Bolt".to_string(),
            &model_file_path,
            ModelType::Interpolate {
                underlying_model_type: Box::new(ModelType::Smartcore),
                speed_lower_bound: Speed::new(0.0),
                speed_upper_bound: Speed::new(100.0),
                speed_bins: 101,
                grade_lower_bound: Grade::new(-0.20),
                grade_upper_bound: Grade::new(0.20),
                grade_bins: 41,
            },
            SpeedUnit::MilesPerHour,
            GradeUnit::Decimal,
            EnergyRateUnit::KilowattHoursPerMile,
            Some(EnergyRate::new(0.2)),
            Some(1.3958),
            None,
        )
        .unwrap();

        let battery_capacity = Energy::new(60.0);
        let staring_battery_energy = battery_capacity * (starting_soc_percent / 100.0);

        BEV::new(
            "Chevy_Bolt".to_string(),
            model_record,
            battery_capacity,
            staring_battery_energy,
            EnergyUnit::KilowattHours,
        )
    }

    #[test]
    fn test_bev_energy_model() {
        let vehicle = mock_vehicle(100.0);
        let initial = vehicle.initial_state();

        // starting at 100% SOC, we should be able to traverse a flat 110 miles at 60 mph
        // and it should use about half of the battery since the EPA range is 238 miles
        let distance = (Distance::new(110.0), DistanceUnit::Miles);
        let speed = (Speed::new(60.0), SpeedUnit::MilesPerHour);
        let grade = (Grade::new(0.0), GradeUnit::Decimal);

        let result = vehicle
            .consume_energy(speed, grade, distance, &initial)
            .unwrap();

        let electrical_energy = get_electrical_energy_from_state(&result.updated_state);
        assert!(electrical_energy.as_f64() > 0.0);

        let battery_percent_soc = get_battery_soc_percent(&vehicle, &result.updated_state);
        assert!(battery_percent_soc < 60.0);
        assert!(battery_percent_soc > 40.0);
    }

    #[test]
    fn test_bev_energy_model_regen() {
        let vehicle = mock_vehicle(20.0);
        let initial = vehicle.initial_state();

        // starting at 20% SOC, going downhill at -5% grade for 10 miles at 55mph, we should be see
        // some regen braking events and should end up with more energy than we started with
        let distance = (Distance::new(10.0), DistanceUnit::Miles);
        let speed = (Speed::new(55.0), SpeedUnit::MilesPerHour);
        let grade = (Grade::new(-5.0), GradeUnit::Percent);

        let result = vehicle
            .consume_energy(speed, grade, distance, &initial)
            .unwrap();

        let electrical_energy = get_electrical_energy_from_state(&result.updated_state);
        assert!(electrical_energy.as_f64() < 0.0);

        let battery_percent_soc = get_battery_soc_percent(&vehicle, &result.updated_state);
        assert!(battery_percent_soc > 20.0);
        assert!(battery_percent_soc < 30.0);
    }

    #[test]
    fn test_bev_battery_in_bounds_upper() {
        // starting at 100% SOC, even going downhill with regen, we shouldn't be able to exceed 100%
        let vehicle = mock_vehicle(100.0);
        let initial = vehicle.initial_state();

        let distance = (Distance::new(10.0), DistanceUnit::Miles);
        let speed = (Speed::new(55.0), SpeedUnit::MilesPerHour);
        let grade = (Grade::new(-5.0), GradeUnit::Percent);

        let result = vehicle
            .consume_energy(speed, grade, distance, &initial)
            .unwrap();

        let battery_percent_soc = get_battery_soc_percent(&vehicle, &result.updated_state);
        assert!(battery_percent_soc <= 100.0);
    }

    #[test]
    fn test_bev_battery_in_bounds_lower() {
        // starting at 1% SOC, even going uphill, we shouldn't be able to go below 0%
        let vehicle = mock_vehicle(1.0);
        let initial = vehicle.initial_state();

        let distance = (Distance::new(100.0), DistanceUnit::Miles);
        let speed = (Speed::new(55.0), SpeedUnit::MilesPerHour);
        let grade = (Grade::new(5.0), GradeUnit::Percent);

        let result = vehicle
            .consume_energy(speed, grade, distance, &initial)
            .unwrap();

        let battery_percent_soc = get_battery_soc_percent(&vehicle, &result.updated_state);
        assert!(battery_percent_soc >= 0.0);
    }
}
