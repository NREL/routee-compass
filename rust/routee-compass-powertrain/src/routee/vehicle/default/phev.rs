use crate::routee::{prediction::PredictionModelRecord, vehicle::vehicle_type::VehicleType};
use routee_compass_core::model::{
    state::{
        custom_feature_format::CustomFeatureFormat, state_feature::StateFeature,
        state_model::StateModel,
    },
    traversal::{state::state_variable::StateVar, traversal_model_error::TraversalModelError},
    unit::{
        as_f64::AsF64, Distance, DistanceUnit, Energy, EnergyUnit, Grade, GradeUnit, Speed,
        SpeedUnit,
    },
};
use std::sync::Arc;

pub struct PHEV {
    pub name: String,
    pub charge_sustain_model: Arc<PredictionModelRecord>,
    pub charge_depleting_model: Arc<PredictionModelRecord>,
    pub battery_capacity: Energy,
    pub starting_battery_energy: Energy,
    pub battery_energy_unit: EnergyUnit,
    pub custom_liquid_fuel_to_kwh: Option<f64>,
}

impl PHEV {
    const LIQUID_FEATURE_NAME: &'static str = "energy_liquid";
    const ELECTRIC_FEATURE_NAME: &'static str = "energy_electric";
    const SOC_FEATURE_NAME: &'static str = "battery_state";

    pub fn new(
        name: String,
        charge_sustain_model: PredictionModelRecord,
        charge_depleting_model: PredictionModelRecord,
        battery_capacity: Energy,
        starting_battery_energy: Energy,
        battery_energy_unit: EnergyUnit,
        custom_liquid_fuel_to_kwh: Option<f64>,
    ) -> Result<Self, TraversalModelError> {
        Ok(Self {
            name,
            charge_sustain_model: Arc::new(charge_sustain_model),
            charge_depleting_model: Arc::new(charge_depleting_model),
            battery_capacity,
            starting_battery_energy,
            battery_energy_unit,
            custom_liquid_fuel_to_kwh,
        })
    }

    fn as_soc_percent(&self, energy: Energy) -> f64 {
        (energy.as_f64() / self.battery_capacity.as_f64()) * 100.0
    }
}

impl VehicleType for PHEV {
    fn name(&self) -> String {
        self.name.clone()
    }

    fn state_features(&self) -> Vec<(String, StateFeature)> {
        let initial_soc = self.as_soc_percent(self.starting_battery_energy);
        let liquid_energy_unit = self
            .charge_sustain_model
            .energy_rate_unit
            .associated_energy_unit();
        vec![
            (
                String::from(PHEV::ELECTRIC_FEATURE_NAME),
                StateFeature::Electric {
                    energy_electric_unit: self.battery_energy_unit,
                    initial: Energy::ZERO,
                },
            ),
            (
                String::from(PHEV::SOC_FEATURE_NAME),
                StateFeature::Custom {
                    name: String::from("soc"),
                    unit: String::from("percent"),
                    format: CustomFeatureFormat::FloatingPoint {
                        initial: initial_soc,
                    },
                },
            ),
            (
                String::from(PHEV::LIQUID_FEATURE_NAME),
                StateFeature::Liquid {
                    energy_liquid_unit: liquid_energy_unit,
                    initial: Energy::ZERO,
                },
            ),
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

    fn best_case_energy_state(
        &self,
        distance: (Distance, DistanceUnit),
        state: &mut Vec<StateVar>,
        state_model: &StateModel,
    ) -> Result<(), TraversalModelError> {
        let (electrical_energy, _) = self.best_case_energy(distance)?;
        state_model.update_add_bounded(
            state,
            PHEV::ELECTRIC_FEATURE_NAME,
            &electrical_energy.into(),
            &StateVar::ZERO,
            &self.battery_capacity.into(),
        )?;
        Ok(())
    }

    fn consume_energy(
        &self,
        speed: (Speed, SpeedUnit),
        grade: (Grade, GradeUnit),
        distance: (Distance, DistanceUnit),
        state: &mut Vec<StateVar>,
        state_model: &StateModel,
    ) -> Result<(), TraversalModelError> {
        let soc = state_model.get_value(state, PHEV::SOC_FEATURE_NAME)?.into();
        let (electrical_energy, _, liquid_fuel_energy, _) =
            get_phev_energy(self, soc, speed, grade, distance)?;

        state_model.update_add(
            state,
            PHEV::ELECTRIC_FEATURE_NAME,
            &electrical_energy.into(),
        )?;

        // update state of charge (SOC). energy has inverse relationship with SOC.
        let soc_diff_percent = StateVar(-self.as_soc_percent(electrical_energy));
        state_model.update_add_bounded(
            state,
            PHEV::SOC_FEATURE_NAME,
            &soc_diff_percent,
            &StateVar::ZERO,
            &StateVar::ONE_HUNDRED,
        )?;

        state_model.update_add(state, PHEV::LIQUID_FEATURE_NAME, &liquid_fuel_energy.into())?;

        Ok(())
    }

    fn update_from_query(
        &self,
        query: &serde_json::Value,
    ) -> Result<Arc<dyn VehicleType>, TraversalModelError> {
        let starting_soc_percent = query
            .get("starting_soc_percent".to_string())
            .ok_or_else(|| {
                TraversalModelError::BuildError(
                    "No 'starting_soc_percent' key provided in query".to_string(),
                )
            })?
            .as_f64()
            .ok_or_else(|| {
                TraversalModelError::BuildError(
                    "Expected 'starting_soc_percent' value to be numeric".to_string(),
                )
            })?;
        if !(0.0..=100.0).contains(&starting_soc_percent) {
            return Err(TraversalModelError::BuildError(
                "Expected 'starting_soc_percent' value to be between 0 and 100".to_string(),
            ));
        }
        let soc_percent = self.as_soc_percent(Energy::new(starting_soc_percent));
        let starting_battery_energy = Energy::new(soc_percent);

        let new_phev = PHEV {
            name: self.name.clone(),
            charge_sustain_model: self.charge_sustain_model.clone(),
            charge_depleting_model: self.charge_depleting_model.clone(),
            battery_capacity: self.battery_capacity,
            starting_battery_energy,
            battery_energy_unit: self.battery_energy_unit,
            custom_liquid_fuel_to_kwh: self.custom_liquid_fuel_to_kwh,
        };

        Ok(Arc::new(new_phev))
    }
}

/// Compute the energy for the PHEV by converting liquid_fuel to kWh.
/// This uses a simplified operation in which we assume that if the battery
/// SOC is greater than zero we can just operate on battery to traverse a link.
/// This is not entirely realistic as it's possible to arrive at a link with
/// 0.001% SOC and still need to use liquid_fuel to traverse the link.
///
/// In the future we could make this more sophisticated by calculating
/// the energy required to traverse the link using the battery and then
/// finding the point at which we would have to switch to liquid_fuel
///
/// Returns a tuple of (electrical_energy, electrical_energy_unit, liquid_fuel_energy, liquid_fuel_energy_unit)
fn get_phev_energy(
    vehicle: &PHEV,
    battery_soc_percent: f64,
    speed: (Speed, SpeedUnit),
    grade: (Grade, GradeUnit),
    distance: (Distance, DistanceUnit),
) -> Result<(Energy, EnergyUnit, Energy, EnergyUnit), TraversalModelError> {
    let electrical_energy_unit = vehicle
        .charge_depleting_model
        .energy_rate_unit
        .associated_energy_unit();
    let liquid_fuel_energy_unit = vehicle
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
            liquid_fuel_energy_unit,
        ))
    } else {
        // just use the liquid_fuel engine
        let (liquid_fuel_energy, liquid_fuel_energy_unit) = vehicle
            .charge_sustain_model
            .predict(speed, grade, distance)?;
        Ok((
            Energy::new(0.0),
            electrical_energy_unit,
            liquid_fuel_energy,
            liquid_fuel_energy_unit,
        ))
    }
}

#[cfg(test)]
mod tests {
    use routee_compass_core::model::unit::{EnergyRate, EnergyRateUnit};

    use crate::routee::{prediction::load_prediction_model, prediction::model_type::ModelType};

    use super::*;

    use std::path::PathBuf;

    fn mock_vehicle() -> PHEV {
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
        let model_type = ModelType::Interpolate {
            underlying_model_type: Box::new(ModelType::Smartcore),
            speed_lower_bound: Speed::new(0.0),
            speed_upper_bound: Speed::new(100.0),
            speed_bins: 101,
            grade_lower_bound: Grade::new(-0.20),
            grade_upper_bound: Grade::new(0.20),
            grade_bins: 41,
        };

        let charge_sustain_model_record = load_prediction_model(
            "Chevy_Volt_Charge_Sustaining".to_string(),
            &charge_sustain_model_file_path,
            model_type.clone(),
            SpeedUnit::MilesPerHour,
            GradeUnit::Decimal,
            EnergyRateUnit::GallonsGasolinePerMile,
            Some(EnergyRate::new(0.02)),
            Some(1.1252),
            None,
        )
        .unwrap();
        let charge_depleting_model_record = load_prediction_model(
            "Chevy_Volt_Charge_Depleting".to_string(),
            &charge_depleting_model_file_path,
            model_type.clone(),
            SpeedUnit::MilesPerHour,
            GradeUnit::Decimal,
            EnergyRateUnit::KilowattHoursPerMile,
            Some(EnergyRate::new(0.2)),
            Some(1.3958),
            None,
        )
        .unwrap();

        PHEV::new(
            "Chevy_Volt".to_string(),
            charge_sustain_model_record,
            charge_depleting_model_record,
            Energy::new(12.0),
            Energy::new(12.0),
            EnergyUnit::KilowattHours,
            None,
        )
        .unwrap()
    }

    #[test]
    fn test_phev_energy_model_just_electric() {
        let vehicle = mock_vehicle();
        let state_model = StateModel::empty()
            .extend(vehicle.state_features())
            .unwrap();
        let mut state = state_model.initial_state().unwrap();

        // starting at 100% SOC, we should be able to traverse 1000 meters
        // without using any liquid_fuel
        let distance = (Distance::new(1000.0), DistanceUnit::Meters);
        let speed = (Speed::new(60.0), SpeedUnit::MilesPerHour);
        let grade = (Grade::new(0.0), GradeUnit::Decimal);

        vehicle
            .consume_energy(speed, grade, distance, &mut state, &state_model)
            .unwrap();

        let elec = state_model
            .get_value(&state, PHEV::ELECTRIC_FEATURE_NAME)
            .unwrap();
        assert!(elec.0 > 0.0, "elec energy {} should be > 0", elec);

        let liquid = state_model
            .get_value(&state, PHEV::LIQUID_FEATURE_NAME)
            .unwrap();
        assert!(
            liquid.0 < 1e-9,
            "liquid energy {} should be miniscule, < {}",
            liquid,
            1e-9
        );

        let soc = state_model
            .get_value(&state, PHEV::SOC_FEATURE_NAME)
            .unwrap();
        assert!(soc.0 < 100.0, "soc {} should be < 100%", soc);
    }

    #[test]
    fn test_phev_energy_model_gas_and_electric() {
        let vehicle = mock_vehicle();
        let state_model = StateModel::empty()
            .extend(vehicle.state_features())
            .unwrap();
        let mut state = state_model.initial_state().unwrap();

        // now let's traverse a really long link to deplete the battery
        let distance = (Distance::new(100.0), DistanceUnit::Miles);
        let speed = (Speed::new(60.0), SpeedUnit::MilesPerHour);
        let grade = (Grade::new(0.0), GradeUnit::Decimal);

        vehicle
            .consume_energy(speed, grade, distance, &mut state, &state_model)
            .unwrap();

        let elec = state_model
            .get_value(&state, PHEV::ELECTRIC_FEATURE_NAME)
            .unwrap();
        let soc = state_model
            .get_value(&state, PHEV::SOC_FEATURE_NAME)
            .unwrap();
        let liquid = state_model
            .get_value(&state, PHEV::LIQUID_FEATURE_NAME)
            .unwrap();

        assert!(elec > StateVar::ZERO, "elec energy {} should be > 0", elec);
        assert!(soc.0 < 1e-9, "soc {} should be miniscule, < {}", soc, 1e-9);
        assert!(
            liquid == StateVar::ZERO,
            "should not have used liquid energy"
        );

        // and then traverse the same distance but this time we should only use liquid_fuel energy
        vehicle
            .consume_energy(speed, grade, distance, &mut state, &state_model)
            .unwrap();

        let liquid_energy_2 = state_model
            .get_value(&state, PHEV::LIQUID_FEATURE_NAME)
            .unwrap();

        assert!(liquid_energy_2 > StateVar::ZERO);
    }
}
