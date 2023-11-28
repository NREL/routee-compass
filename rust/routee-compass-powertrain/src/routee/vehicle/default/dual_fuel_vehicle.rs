use std::sync::Arc;

use routee_compass_core::{
    model::traversal::{
        state::{state_variable::StateVar, traversal_state::TraversalState},
        traversal_model_error::TraversalModelError,
    },
    util::unit::{
        as_f64::AsF64, Distance, DistanceUnit, Energy, EnergyUnit, Grade, GradeUnit, Speed,
        SpeedUnit,
    },
};

use crate::routee::{
    prediction::PredictionModelRecord,
    vehicle::{vehicle_type::VehicleType, VehicleEnergyResult},
};

pub struct DualFuelVehicle {
    pub name: String,
    pub charge_sustain_model: Arc<PredictionModelRecord>,
    pub charge_depleting_model: Arc<PredictionModelRecord>,
    pub battery_capacity: Energy,
    pub starting_battery_energy: Energy,
    pub battery_energy_unit: EnergyUnit,
}

impl DualFuelVehicle {
    pub fn new(
        name: String,
        charge_sustain_model: PredictionModelRecord,
        charge_depleting_model: PredictionModelRecord,
        battery_capacity: Energy,
        starting_battery_energy: Energy,
        battery_energy_unit: EnergyUnit,
    ) -> Result<Self, TraversalModelError> {
        Ok(Self {
            name,
            charge_sustain_model: Arc::new(charge_sustain_model),
            charge_depleting_model: Arc::new(charge_depleting_model),
            battery_capacity,
            starting_battery_energy,
            battery_energy_unit,
        })
    }
}

impl VehicleType for DualFuelVehicle {
    fn name(&self) -> String {
        self.name.clone()
    }
    fn initial_state(&self) -> TraversalState {
        vec![
            StateVar(0.0),                                   // accumulated electrical energy
            StateVar(0.0),                                   // accumulated gasoline energy
            StateVar(self.starting_battery_energy.as_f64()), // battery energy remaining
        ]
    }
    fn best_case_energy(
        &self,
        distance: (Distance, DistanceUnit),
    ) -> Result<(Energy, EnergyUnit), TraversalModelError> {
        let (distance, distance_unit) = distance;

        // assume lowest energy cost scenario for a PHEV is to just use the battery
        let energy = Energy::create(
            self.charge_depleting_model.ideal_energy_rate,
            self.charge_depleting_model.energy_rate_unit,
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
        let battery_soc_percentage = get_battery_soc_percent(self, state);

        let (electrical_energy, electrical_energy_unit, gasoline_energy, gasoline_energy_unit) =
            get_phev_energy(self, battery_soc_percentage, speed, grade, distance)?;

        // convert both energy sources to kWh
        let electrical_energy_kwh =
            electrical_energy_unit.convert(electrical_energy, EnergyUnit::KilowattHours);
        let gasoline_energy_kwh =
            gasoline_energy_unit.convert(gasoline_energy, EnergyUnit::KilowattHours);
        let total_energy_kwh = electrical_energy_kwh + gasoline_energy_kwh;

        let updated_state = update_state(state, electrical_energy, gasoline_energy);

        Ok(VehicleEnergyResult {
            energy: total_energy_kwh,
            energy_unit: EnergyUnit::KilowattHours,
            updated_state,
        })
    }
    fn serialize_state(&self, state: &[StateVar]) -> serde_json::Value {
        let battery_energy = get_electrical_energy_from_state(state);
        let gasoline_energy = get_gasoline_energy_from_state(state);
        let battery_soc_percent = get_battery_soc_percent(self, state);
        serde_json::json!({
            "battery_energy": battery_energy.as_f64(),
            "fuel_energy": gasoline_energy.as_f64(),
            "battery_soc_percent": battery_soc_percent,
        })
    }

    fn serialize_state_info(&self, _state: &[StateVar]) -> serde_json::Value {
        let battery_energy_unit = self.battery_energy_unit;
        let fuel_energy_unit = self
            .charge_sustain_model
            .energy_rate_unit
            .associated_energy_unit();
        serde_json::json!({
            "battery_energy_unit": battery_energy_unit.to_string(),
            "fuel_energy_unit": fuel_energy_unit.to_string(),
        })
    }

    fn update_from_query(
        &self,
        query: &serde_json::Value,
    ) -> Result<Arc<dyn VehicleType>, TraversalModelError> {
        let starting_soc_percent = query
            .get("starting_soc_percent".to_string())
            .ok_or(TraversalModelError::BuildError(
                "No 'starting_soc_percent' key provided in query".to_string(),
            ))?
            .as_f64()
            .ok_or(TraversalModelError::BuildError(
                "Expected 'starting_soc_percent' value to be numeric".to_string(),
            ))?;
        if !(0.0..=100.0).contains(&starting_soc_percent) {
            return Err(TraversalModelError::BuildError(
                "Expected 'starting_soc_percent' value to be between 0 and 100".to_string(),
            ));
        }
        let starting_battery_energy = self.battery_capacity * (starting_soc_percent / 100.0);

        let new_phev = DualFuelVehicle {
            name: self.name.clone(),
            charge_sustain_model: self.charge_sustain_model.clone(),
            charge_depleting_model: self.charge_depleting_model.clone(),
            battery_capacity: self.battery_capacity,
            starting_battery_energy,
            battery_energy_unit: self.battery_energy_unit,
        };

        Ok(Arc::new(new_phev))
    }
}

fn update_state(
    state: &[StateVar],
    electrical_energy: Energy,
    gasoline_energy: Energy,
) -> TraversalState {
    let mut updated_state = Vec::with_capacity(state.len());

    // accumulated electrical energy
    updated_state.push(state[0] + electrical_energy.into());

    // accumulated fuel energy
    updated_state.push(state[1] + gasoline_energy.into());

    // remaining battery energy
    let current_battery_energy = get_remaining_battery_energy_from_state(state);
    let new_battery_energy = (current_battery_energy - electrical_energy).max(Energy::new(0.0));
    updated_state.push(new_battery_energy.into());

    updated_state
}
fn get_electrical_energy_from_state(state: &[StateVar]) -> Energy {
    Energy::new(state[0].0)
}

fn get_gasoline_energy_from_state(state: &[StateVar]) -> Energy {
    Energy::new(state[1].0)
}

fn get_remaining_battery_energy_from_state(state: &[StateVar]) -> Energy {
    Energy::new(state[2].0)
}

fn get_battery_soc_percent(vehicle: &DualFuelVehicle, state: &[StateVar]) -> f64 {
    let battery_energy_unit = vehicle.battery_energy_unit;

    let battery_capacity_kwh =
        battery_energy_unit.convert(vehicle.battery_capacity, EnergyUnit::KilowattHours);

    let remaining_battery_energy = get_remaining_battery_energy_from_state(state);

    let remaining_battery_energy_kwh =
        battery_energy_unit.convert(remaining_battery_energy, EnergyUnit::KilowattHours);

    (remaining_battery_energy_kwh.as_f64() / battery_capacity_kwh.as_f64()) * 100.0
}

/// Compute the energy for the PHEV by converting gasoline to kWh.
/// This uses a simplified operation in which we assume that if the battery
/// SOC is greater than zero we can just operate on battery to traverse a link.
/// This is not entirely realistic as it's possible to arrive at a link with
/// 0.001% SOC and still need to use gasoline to traverse the link.
///
/// In the future we could make this more sophisticated by calculating
/// the energy required to traverse the link using the battery and then
/// finding the point at which we would have to switch to gasoline
///
/// Returns a tuple of (electrical_energy, electrical_energy_unit, gasoline_energy, gasoline_energy_unit)
fn get_phev_energy(
    vehicle: &DualFuelVehicle,
    battery_soc_percent: f64,
    speed: (Speed, SpeedUnit),
    grade: (Grade, GradeUnit),
    distance: (Distance, DistanceUnit),
) -> Result<(Energy, EnergyUnit, Energy, EnergyUnit), TraversalModelError> {
    let electrical_energy_unit = vehicle
        .charge_depleting_model
        .energy_rate_unit
        .associated_energy_unit();
    let gasoline_energy_unit = vehicle
        .charge_sustain_model
        .energy_rate_unit
        .associated_energy_unit();

    if battery_soc_percent > 0.0 {
        // assume we can just use the battery
        let (electrical_energy, electrical_energy_unit) = vehicle
            .charge_depleting_model
            .predict(speed, grade, distance)?;
        Ok((
            electrical_energy,
            electrical_energy_unit,
            Energy::new(0.0),
            gasoline_energy_unit,
        ))
    } else {
        // just use the gasoline engine
        let (gasoline_energy, gasoline_energy_unit) = vehicle
            .charge_sustain_model
            .predict(speed, grade, distance)?;
        Ok((
            Energy::new(0.0),
            electrical_energy_unit,
            gasoline_energy,
            gasoline_energy_unit,
        ))
    }
}

#[cfg(test)]
mod tests {
    use routee_compass_core::util::unit::{EnergyRate, EnergyRateUnit};

    use crate::routee::{prediction::load_prediction_model, prediction::model_type::ModelType};

    use super::*;

    use std::path::PathBuf;

    fn mock_vehicle() -> DualFuelVehicle {
        let charge_sustain_model_file_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join("routee")
            .join("test")
            .join("2016_CHEVROLET_Volt_Charge_Sustaining.bin");
        let charge_depleting_model_file_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join("routee")
            .join("test")
            .join("2016_CHEVROLET_Volt_Charge_Depleting.bin");

        let charge_sustain_model_record = load_prediction_model(
            "Chevy_Volt_Charge_Sustaining".to_string(),
            &charge_sustain_model_file_path,
            ModelType::Smartcore,
            SpeedUnit::MilesPerHour,
            GradeUnit::Decimal,
            EnergyRateUnit::GallonsGasolinePerMile,
            Some(EnergyRate::new(0.02)),
            Some(1.1252),
        )
        .unwrap();
        let charge_depleting_model_record = load_prediction_model(
            "Chevy_Volt_Charge_Depleting".to_string(),
            &charge_depleting_model_file_path,
            ModelType::Smartcore,
            SpeedUnit::MilesPerHour,
            GradeUnit::Decimal,
            EnergyRateUnit::KilowattHoursPerMile,
            Some(EnergyRate::new(0.2)),
            Some(1.3958),
        )
        .unwrap();

        DualFuelVehicle::new(
            "Chevy_Volt".to_string(),
            charge_sustain_model_record,
            charge_depleting_model_record,
            Energy::new(12.0),
            Energy::new(12.0),
            EnergyUnit::KilowattHours,
        )
        .unwrap()
    }

    #[test]
    fn test_phev_energy_model_just_electric() {
        let vehicle = mock_vehicle();
        let initial = vehicle.initial_state();

        // starting at 100% SOC, we should be able to traverse 1000 meters
        // without using any gasoline
        let distance = (Distance::new(1000.0), DistanceUnit::Meters);
        let speed = (Speed::new(60.0), SpeedUnit::MilesPerHour);
        let grade = (Grade::new(0.0), GradeUnit::Decimal);

        let result = vehicle
            .consume_energy(speed, grade, distance, &initial)
            .unwrap();

        let gasoline_energy = get_gasoline_energy_from_state(&result.updated_state);
        assert!(gasoline_energy.as_f64() < 1e-9);

        let electrical_energy = get_electrical_energy_from_state(&result.updated_state);
        assert!(electrical_energy.as_f64() > 0.0);

        let battery_percent_soc = get_battery_soc_percent(&vehicle, &result.updated_state);
        assert!(battery_percent_soc < 100.0);
    }

    #[test]
    fn test_phev_energy_model_gas_and_electric() {
        let vehicle = mock_vehicle();
        let initial = vehicle.initial_state();

        // now let's traverse a really long link to deplete the battery
        let distance = (Distance::new(100.0), DistanceUnit::Miles);
        let speed = (Speed::new(60.0), SpeedUnit::MilesPerHour);
        let grade = (Grade::new(0.0), GradeUnit::Decimal);

        let result = vehicle
            .consume_energy(speed, grade, distance, &initial)
            .unwrap();

        let electrical_energy = get_electrical_energy_from_state(&result.updated_state);
        let battery_percent_soc = get_battery_soc_percent(&vehicle, &result.updated_state);

        assert!(electrical_energy.as_f64() > 0.0);
        assert!(battery_percent_soc < 1e-9);

        // and then traverse the same distance but this time we should only use gasoline energy
        let result2 = vehicle
            .consume_energy(speed, grade, distance, &result.updated_state)
            .unwrap();

        let gasoline_energy = get_gasoline_energy_from_state(&result2.updated_state);

        assert!(gasoline_energy.as_f64() > 0.0);
    }
}
