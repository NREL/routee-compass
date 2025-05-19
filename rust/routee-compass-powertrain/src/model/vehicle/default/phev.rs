use crate::model::{
    prediction::PredictionModelRecord,
    vehicle::{vehicle_ops, vehicle_type::VehicleType},
};
use routee_compass_core::model::{
    state::{CustomFeatureFormat, StateFeature, StateModel, StateVariable},
    traversal::TraversalModelError,
    unit::{
        AsF64, Convert, Distance, DistanceUnit, Energy, EnergyUnit, Grade, GradeUnit, Speed,
        SpeedUnit,
    },
};
use std::{borrow::Cow, sync::Arc};

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
}

impl VehicleType for PHEV {
    fn name(&self) -> String {
        self.name.clone()
    }

    fn state_features(&self) -> Vec<(String, StateFeature)> {
        let initial_soc =
            vehicle_ops::as_soc_percent(&self.starting_battery_energy, &self.battery_capacity);
        let liquid_energy_unit = self
            .charge_sustain_model
            .energy_rate_unit
            .associated_energy_unit();
        vec![
            (
                String::from(PHEV::ELECTRIC_FEATURE_NAME),
                StateFeature::Energy {
                    energy_unit: self.battery_energy_unit,
                    initial: Energy::ZERO,
                },
            ),
            (
                String::from(PHEV::SOC_FEATURE_NAME),
                StateFeature::Custom {
                    r#type: String::from("soc"),
                    unit: String::from("percent"),
                    format: CustomFeatureFormat::FloatingPoint {
                        initial: initial_soc.into(),
                    },
                },
            ),
            (
                String::from(PHEV::LIQUID_FEATURE_NAME),
                StateFeature::Energy {
                    energy_unit: liquid_energy_unit,
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
            (&distance, &distance_unit),
            (
                &self.charge_depleting_model.ideal_energy_rate,
                &self.charge_depleting_model.energy_rate_unit,
            ),
        )?;
        Ok(energy)
    }

    fn best_case_energy_state(
        &self,
        distance: (Distance, DistanceUnit),
        state: &mut Vec<StateVariable>,
        state_model: &StateModel,
    ) -> Result<(), TraversalModelError> {
        let (electrical_energy, _) = self.best_case_energy(distance)?;
        state_model.add_energy(
            state,
            &PHEV::ELECTRIC_FEATURE_NAME.into(),
            &electrical_energy,
            &self.battery_energy_unit,
        )?;
        vehicle_ops::update_soc_percent(
            state,
            PHEV::SOC_FEATURE_NAME,
            &electrical_energy,
            &self.battery_capacity,
            state_model,
        )?;
        Ok(())
    }

    fn consume_energy(
        &self,
        speed: (Speed, SpeedUnit),
        grade: (Grade, GradeUnit),
        distance: (Distance, DistanceUnit),
        state: &mut Vec<StateVariable>,
        state_model: &StateModel,
    ) -> Result<(), TraversalModelError> {
        let start_soc = state_model.get_custom_f64(state, &PHEV::SOC_FEATURE_NAME.into())?;
        let (elec_energy, elec_unit, liq_energy, liq_unit) =
            get_phev_energy(self, start_soc, speed, grade, distance)?;

        state_model.add_energy(
            state,
            &PHEV::ELECTRIC_FEATURE_NAME.into(),
            &elec_energy,
            &elec_unit,
        )?;
        state_model.add_energy(
            state,
            &PHEV::LIQUID_FEATURE_NAME.into(),
            &liq_energy,
            &liq_unit,
        )?;
        let mut delta = Cow::Owned(elec_energy);
        elec_unit.convert(&mut delta, &self.battery_energy_unit)?;
        vehicle_ops::update_soc_percent(
            state,
            PHEV::SOC_FEATURE_NAME,
            &delta,
            &self.battery_capacity,
            state_model,
        )?;

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
        let starting_battery_energy =
            Energy::from(0.01 * starting_soc_percent * self.battery_capacity.as_f64());

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
            Energy::ZERO,
            liquid_fuel_energy_unit,
        ))
    } else {
        // just use the liquid_fuel engine
        let (liquid_fuel_energy, liquid_fuel_energy_unit) = vehicle
            .charge_sustain_model
            .predict(speed, grade, distance)?;
        Ok((
            Energy::ZERO,
            electrical_energy_unit,
            liquid_fuel_energy,
            liquid_fuel_energy_unit,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::prediction::{load_prediction_model, ModelType};
    use routee_compass_core::model::unit::{AsF64, EnergyRate, EnergyRateUnit, VolumeUnit};
    use std::path::PathBuf;

    fn mock_vehicle() -> PHEV {
        let charge_sustain_model_file_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join("model")
            .join("test")
            .join("2016_CHEVROLET_Volt_Charge_Sustaining.bin");
        let charge_depleting_model_file_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join("model")
            .join("test")
            .join("2016_CHEVROLET_Volt_Charge_Depleting.bin");
        let model_type = ModelType::Interpolate {
            underlying_model_type: Box::new(ModelType::Smartcore),
            speed_lower_bound: Speed::from(0.0),
            speed_upper_bound: Speed::from(100.0),
            speed_bins: 101,
            grade_lower_bound: Grade::from(-0.20),
            grade_upper_bound: Grade::from(0.20),
            grade_bins: 41,
        };

        let charge_sustain_model_record = load_prediction_model(
            "Chevy_Volt_Charge_Sustaining".to_string(),
            &charge_sustain_model_file_path,
            model_type.clone(),
            SpeedUnit::MPH,
            GradeUnit::Decimal,
            EnergyRateUnit::GGPM,
            Some(EnergyRate::from(0.02)),
            Some(1.1252),
            None,
        )
        .unwrap();
        let charge_depleting_model_record = load_prediction_model(
            "Chevy_Volt_Charge_Depleting".to_string(),
            &charge_depleting_model_file_path,
            model_type.clone(),
            SpeedUnit::MPH,
            GradeUnit::Decimal,
            EnergyRateUnit::KWHPM,
            Some(EnergyRate::from(0.2)),
            Some(1.3958),
            None,
        )
        .unwrap();

        PHEV::new(
            "Chevy_Volt".to_string(),
            charge_sustain_model_record,
            charge_depleting_model_record,
            Energy::from(12.0),
            Energy::from(12.0),
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
        let distance = (Distance::from(1000.0), DistanceUnit::Meters);
        let speed = (Speed::from(60.0), SpeedUnit::MPH);
        let grade = (Grade::from(0.0), GradeUnit::Decimal);

        vehicle
            .consume_energy(speed, grade, distance, &mut state, &state_model)
            .unwrap();

        let elec = state_model
            .get_energy(
                &state,
                &PHEV::ELECTRIC_FEATURE_NAME.into(),
                &EnergyUnit::KilowattHours,
            )
            .unwrap();
        assert!(elec.as_f64() > 0.0, "elec energy {} should be > 0", elec);

        let liquid = state_model
            .get_energy(
                &state,
                &PHEV::LIQUID_FEATURE_NAME.into(),
                &EnergyUnit::Gasoline(VolumeUnit::GallonsUs),
            )
            .unwrap();
        assert!(
            liquid.as_f64() < 1e-9,
            "liquid energy {} should be miniscule, < {}",
            liquid,
            1e-9
        );

        let soc = state_model
            .get_custom_f64(&state, &PHEV::SOC_FEATURE_NAME.into())
            .unwrap();
        assert!(soc < 100.0, "soc {} should be < 100%", soc);
    }

    #[test]
    fn test_phev_energy_model_gas_and_electric() {
        let vehicle = mock_vehicle();
        let state_model = StateModel::empty()
            .extend(vehicle.state_features())
            .unwrap();
        let mut state = state_model.initial_state().unwrap();

        // now let's traverse a really long link to deplete the battery
        let distance = (Distance::from(100.0), DistanceUnit::Miles);
        let speed = (Speed::from(60.0), SpeedUnit::MPH);
        let grade = (Grade::from(0.0), GradeUnit::Decimal);

        vehicle
            .consume_energy(speed, grade, distance, &mut state, &state_model)
            .unwrap();

        let elec = state_model
            .get_energy(
                &state,
                &PHEV::ELECTRIC_FEATURE_NAME.into(),
                &EnergyUnit::KilowattHours,
            )
            .unwrap();
        let soc = state_model
            .get_custom_f64(&state, &PHEV::SOC_FEATURE_NAME.into())
            .unwrap();
        let liquid = state_model
            .get_energy(
                &state,
                &PHEV::LIQUID_FEATURE_NAME.into(),
                &EnergyUnit::Gasoline(VolumeUnit::GallonsUs),
            )
            .unwrap();

        assert!(elec > Energy::ZERO, "elec energy {} should be > 0", elec);
        assert!(soc < 1e-9, "soc {} should be miniscule, < {}", soc, 1e-9);
        assert!(liquid == Energy::ZERO, "should not have used liquid energy");

        // and then traverse the same distance but this time we should only use liquid_fuel energy
        vehicle
            .consume_energy(speed, grade, distance, &mut state, &state_model)
            .unwrap();

        let liquid_energy_2 = state_model
            .get_energy(
                &state,
                &PHEV::LIQUID_FEATURE_NAME.into(),
                &EnergyUnit::KilowattHours,
            )
            .unwrap();

        assert!(liquid_energy_2 > Energy::ZERO);
    }
}
