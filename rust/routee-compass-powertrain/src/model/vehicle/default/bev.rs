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

pub struct BEV {
    pub name: String,
    pub prediction_model_record: Arc<PredictionModelRecord>,
    pub battery_capacity: Energy,
    pub starting_battery_energy: Energy,
    pub battery_energy_unit: EnergyUnit,
}

impl BEV {
    const ENERGY_FEATURE_NAME: &'static str = "energy_electric";
    const SOC_FEATURE_NAME: &'static str = "battery_state";

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

    fn state_features(&self) -> Vec<(String, StateFeature)> {
        let initial_soc =
            vehicle_ops::as_soc_percent(&self.starting_battery_energy, &self.battery_capacity);
        vec![
            (
                String::from(BEV::ENERGY_FEATURE_NAME),
                StateFeature::Energy {
                    energy_unit: self.battery_energy_unit,
                    initial: Energy::ZERO,
                },
            ),
            (
                String::from(BEV::SOC_FEATURE_NAME),
                StateFeature::Custom {
                    r#type: String::from("soc"),
                    unit: String::from("percent"),
                    format: CustomFeatureFormat::FloatingPoint {
                        initial: initial_soc.into(),
                    },
                },
            ),
        ]
    }

    fn best_case_energy(
        &self,
        distance: (Distance, DistanceUnit),
    ) -> Result<(Energy, EnergyUnit), TraversalModelError> {
        let (distance, distance_unit) = distance;

        let energy = Energy::create(
            (&distance, &distance_unit),
            (
                &self.prediction_model_record.ideal_energy_rate,
                &self.prediction_model_record.energy_rate_unit,
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
        let (energy, _) = self.best_case_energy(distance)?;
        state_model.add_energy(
            state,
            &BEV::ENERGY_FEATURE_NAME.into(),
            &energy,
            &self.battery_energy_unit,
        )?;
        vehicle_ops::update_soc_percent(
            state,
            BEV::SOC_FEATURE_NAME,
            &energy,
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
        let (predicted_energy, energy_unit) = self
            .prediction_model_record
            .predict(speed, grade, distance)?;
        let mut battery_delta = Cow::Owned(predicted_energy);
        energy_unit.convert(&mut battery_delta, &self.battery_energy_unit)?;
        state_model.add_energy(
            state,
            &BEV::ENERGY_FEATURE_NAME.into(),
            &predicted_energy,
            &energy_unit,
        )?;
        vehicle_ops::update_soc_percent(
            state,
            BEV::SOC_FEATURE_NAME,
            &battery_delta,
            &self.battery_capacity,
            state_model,
        )?;

        Ok(())
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

        let starting_battery_energy =
            Energy::from(0.01 * starting_soc_percent * self.battery_capacity.as_f64());

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::prediction::{load_prediction_model, ModelType};
    use routee_compass_core::model::unit::{AsF64, EnergyRate, EnergyRateUnit};
    use std::path::PathBuf;

    fn mock_vehicle(starting_soc_percent: f64) -> BEV {
        let model_file_path: PathBuf = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join("model")
            .join("test")
            .join("2017_CHEVROLET_Bolt.bin");

        let model_record = load_prediction_model(
            "Chevy Bolt".to_string(),
            &model_file_path,
            ModelType::Interpolate {
                underlying_model_type: Box::new(ModelType::Smartcore),
                speed_lower_bound: Speed::from(0.0),
                speed_upper_bound: Speed::from(100.0),
                speed_bins: 101,
                grade_lower_bound: Grade::from(-0.20),
                grade_upper_bound: Grade::from(0.20),
                grade_bins: 41,
            },
            SpeedUnit::MPH,
            GradeUnit::Decimal,
            EnergyRateUnit::KWHPM,
            Some(EnergyRate::from(0.2)),
            Some(1.3958),
            None,
        )
        .unwrap();

        let battery_capacity = Energy::from(60.0);
        let staring_battery_energy: Energy = battery_capacity * (starting_soc_percent / 100.0);

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
        let state_model = StateModel::empty()
            .extend(vehicle.state_features())
            .unwrap();
        let mut state = state_model.initial_state().unwrap();

        // starting at 100% SOC, we should be able to traverse a flat 110 miles at 60 mph
        // and it should use about half of the battery since the EPA range is 238 miles
        let distance = (Distance::from(110.0), DistanceUnit::Miles);
        let speed = (Speed::from(60.0), SpeedUnit::MPH);
        let grade = (Grade::from(0.0), GradeUnit::Decimal);

        vehicle
            .consume_energy(speed, grade, distance, &mut state, &state_model)
            .unwrap();

        let elec = state_model
            .get_energy(
                &state,
                &BEV::ENERGY_FEATURE_NAME.into(),
                &EnergyUnit::KilowattHours,
            )
            .unwrap();
        assert!(elec.as_f64() > 0.0, "elec energy {} should be > 0.0", elec);

        let soc = state_model
            .get_custom_f64(&state, &BEV::SOC_FEATURE_NAME.into())
            .unwrap();

        assert!(soc < 60.0, "soc {} should be < 60.0%", soc);
        assert!(soc > 40.0, "soc {} should be > 40.0%", soc);
    }

    #[test]
    fn test_bev_energy_model_regen() {
        let vehicle = mock_vehicle(20.0);
        let state_model = StateModel::empty()
            .extend(vehicle.state_features())
            .unwrap();
        let mut state = state_model.initial_state().unwrap();

        // starting at 20% SOC, going downhill at -5% grade for 10 miles at 55mph, we should be see
        // some regen braking events and should end up with more energy than we started with
        let distance = (Distance::from(10.0), DistanceUnit::Miles);
        let speed = (Speed::from(55.0), SpeedUnit::MPH);
        let grade = (Grade::from(-5.0), GradeUnit::Percent);

        vehicle
            .consume_energy(speed, grade, distance, &mut state, &state_model)
            .unwrap();

        let elec = state_model
            .get_energy(
                &state,
                &BEV::ENERGY_FEATURE_NAME.into(),
                &EnergyUnit::KilowattHours,
            )
            .unwrap();
        assert!(
            elec.as_f64() < 0.0,
            "elec energy {} should be < 0 (regen)",
            elec
        );

        let soc = state_model
            .get_custom_f64(&state, &BEV::SOC_FEATURE_NAME.into())
            .unwrap();
        assert!(soc > 20.0, "soc {} should be > 20.0", soc);
        assert!(soc < 30.0, "soc {} should be < 30.0", soc);
    }

    #[test]
    fn test_bev_battery_in_bounds_upper() {
        // starting at 100% SOC, even going downhill with regen, we shouldn't be able to exceed 100%
        let vehicle = mock_vehicle(100.0);
        let state_model = StateModel::empty()
            .extend(vehicle.state_features())
            .unwrap();
        let mut state = state_model.initial_state().unwrap();

        let distance = (Distance::from(10.0), DistanceUnit::Miles);
        let speed = (Speed::from(55.0), SpeedUnit::MPH);
        let grade = (Grade::from(-5.0), GradeUnit::Percent);

        vehicle
            .consume_energy(speed, grade, distance, &mut state, &state_model)
            .unwrap();

        let battery_percent_soc = state_model
            .get_custom_f64(&state, &BEV::SOC_FEATURE_NAME.into())
            .unwrap();
        assert!(battery_percent_soc <= 100.0);
    }

    #[test]
    fn test_bev_battery_in_bounds_lower() {
        // starting at 1% SOC, even going uphill, we shouldn't be able to go below 0%
        let vehicle = mock_vehicle(1.0);
        let state_model = StateModel::empty()
            .extend(vehicle.state_features())
            .unwrap();
        let mut state = state_model.initial_state().unwrap();

        let distance = (Distance::from(100.0), DistanceUnit::Miles);
        let speed = (Speed::from(55.0), SpeedUnit::MPH);
        let grade = (Grade::from(5.0), GradeUnit::Percent);

        vehicle
            .consume_energy(speed, grade, distance, &mut state, &state_model)
            .unwrap();

        let battery_percent_soc = state_model
            .get_custom_f64(&state, &BEV::SOC_FEATURE_NAME.into())
            .unwrap();
        assert!(battery_percent_soc >= 0.0);
    }
}
