use super::{
    energy_model_ops,
    prediction::{PredictionModelConfig, PredictionModelRecord},
};
use routee_compass_core::model::{
    network::{Edge, Vertex},
    state::{CustomFeatureFormat, InputFeature, OutputFeature, StateModel, StateVariable},
    traversal::{TraversalModel, TraversalModelError, TraversalModelService},
    unit::{Energy, EnergyUnit},
};
use serde_json::Value;
use std::sync::Arc;

#[derive(Clone)]
pub struct BevEnergyModel {
    prediction_model_record: Arc<PredictionModelRecord>,
    battery_capacity: (Energy, EnergyUnit),
    starting_soc: f64,
}

impl BevEnergyModel {
    const EDGE_ENERGY_ELECTRIC: &'static str = "edge_energy_electric";
    const TRIP_ENERGY_ELECTRIC: &'static str = "trip_energy_electric";
    const EDGE_DISTANCE: &'static str = "edge_distance";
    const EDGE_SPEED: &'static str = "edge_speed";
    const EDGE_GRADE: &'static str = "edge_grade";
    const TRIP_SOC: &'static str = "trip_soc";

    pub fn new(
        prediction_model_record: Arc<PredictionModelRecord>,
        battery_capacity: (Energy, EnergyUnit),
        starting_battery_energy: (Energy, EnergyUnit),
    ) -> Result<Self, TraversalModelError> {
        let starting_energy = (&starting_battery_energy.0, &starting_battery_energy.1);
        let bat_cap_ref = (&battery_capacity.0, &battery_capacity.1);
        let starting_soc = energy_model_ops::soc_from_energy(starting_energy, bat_cap_ref)
            .map_err(TraversalModelError::BuildError)?;
        Ok(Self {
            prediction_model_record,
            battery_capacity,
            starting_soc,
        })
    }
}

impl TraversalModelService for BevEnergyModel {
    fn build(
        &self,
        query: &serde_json::Value,
    ) -> Result<Arc<dyn TraversalModel>, TraversalModelError> {
        let (capacity, capacity_unit) = (self.battery_capacity.0, self.battery_capacity.1);
        match energy_model_ops::get_query_start_energy(query, &capacity)? {
            None => Ok(Arc::new(self.clone())),
            Some(starting_energy) => {
                let updated = Self::new(
                    self.prediction_model_record.clone(),
                    (capacity, capacity_unit),
                    (starting_energy, capacity_unit),
                )?;
                Ok(Arc::new(updated))
            }
        }
    }
}

impl TryFrom<&Value> for BevEnergyModel {
    type Error = TraversalModelError;

    fn try_from(value: &Value) -> Result<Self, Self::Error> {
        let config: PredictionModelConfig = serde_json::from_value(value.clone()).map_err(|e| {
            TraversalModelError::BuildError(format!(
                "failure reading prediction model configuration: {}",
                e
            ))
        })?;
        let prediction_model = PredictionModelRecord::try_from(&config)?;
        let battery_capacity_conf = value.get("battery_capacity").ok_or_else(|| {
            TraversalModelError::BuildError(String::from("missing key 'battery_capacity'"))
        })?;
        let battery_energy_unit_conf = value.get("battery_capacity_unit").ok_or_else(|| {
            TraversalModelError::BuildError(String::from("missing key 'battery_energy_unit'"))
        })?;
        let battery_capacity = serde_json::from_value::<Energy>(battery_capacity_conf.clone())
            .map_err(|e| {
                TraversalModelError::BuildError(format!("failed to parse battery capacity: {}", e))
            })?;
        let battery_energy_unit = serde_json::from_value::<EnergyUnit>(
            battery_energy_unit_conf.clone(),
        )
        .map_err(|e| {
            TraversalModelError::BuildError(format!("failed to parse battery capacity unit: {}", e))
        })?;
        let bev = BevEnergyModel::new(
            Arc::new(prediction_model),
            (battery_capacity, battery_energy_unit),
            (battery_capacity, battery_energy_unit),
        )?;
        Ok(bev)
    }
}

impl TraversalModel for BevEnergyModel {
    fn input_features(&self) -> Vec<(String, InputFeature)> {
        vec![
            (
                String::from(Self::EDGE_DISTANCE),
                InputFeature::Distance(Some(
                    self.prediction_model_record
                        .speed_unit
                        .associated_distance_unit(),
                )),
            ),
            (
                String::from(Self::EDGE_SPEED),
                InputFeature::Speed(Some(self.prediction_model_record.speed_unit)),
            ),
            (
                String::from(Self::EDGE_GRADE),
                InputFeature::Grade(Some(self.prediction_model_record.grade_unit)),
            ),
        ]
    }

    fn output_features(&self) -> Vec<(String, OutputFeature)> {
        let energy_unit = self
            .prediction_model_record
            .energy_rate_unit
            .associated_energy_unit();
        vec![
            (
                String::from(Self::TRIP_ENERGY_ELECTRIC),
                OutputFeature::Energy {
                    energy_unit,
                    initial: Energy::ZERO,
                },
            ),
            (
                String::from(Self::EDGE_ENERGY_ELECTRIC),
                OutputFeature::Energy {
                    energy_unit,
                    initial: Energy::ZERO,
                },
            ),
            (
                String::from(Self::TRIP_SOC),
                OutputFeature::Custom {
                    r#type: String::from("soc"),
                    unit: String::from("Percent"),
                    format: CustomFeatureFormat::FloatingPoint {
                        initial: self.starting_soc.into(),
                    },
                },
            ),
        ]
    }

    fn traverse_edge(
        &self,
        _trajectory: (&Vertex, &Edge, &Vertex),
        state: &mut Vec<StateVariable>,
        state_model: &StateModel,
    ) -> Result<(), TraversalModelError> {
        bev_traversal(
            state,
            state_model,
            self.prediction_model_record.clone(),
            (&self.battery_capacity.0, &self.battery_capacity.1),
        )
    }

    fn estimate_traversal(
        &self,
        _od: (&Vertex, &Vertex),
        state: &mut Vec<StateVariable>,
        state_model: &StateModel,
    ) -> Result<(), TraversalModelError> {
        bev_traversal(
            state,
            state_model,
            self.prediction_model_record.clone(),
            (&self.battery_capacity.0, &self.battery_capacity.1),
        )
    }
}

fn bev_traversal(
    state: &mut [StateVariable],
    state_model: &StateModel,
    prediction_model_record: Arc<PredictionModelRecord>,
    battery_capacity: (&Energy, &EnergyUnit),
) -> Result<(), TraversalModelError> {
    let distance_unit = prediction_model_record
        .speed_unit
        .associated_distance_unit();
    let speed_unit = prediction_model_record.speed_unit;
    let grade_unit = prediction_model_record.grade_unit;

    let (distance, _) =
        state_model.get_distance(state, BevEnergyModel::EDGE_DISTANCE, Some(&distance_unit))?;
    let (speed, _) = state_model.get_speed(state, BevEnergyModel::EDGE_SPEED, Some(&speed_unit))?;
    let (grade, _) = state_model.get_grade(state, BevEnergyModel::EDGE_GRADE, Some(&grade_unit))?;
    let soc = state_model.get_custom_f64(state, BevEnergyModel::TRIP_SOC)?;

    let (energy, energy_unit) = prediction_model_record.predict(
        (speed, &speed_unit),
        (grade, &grade_unit),
        (distance, &distance_unit),
    )?;
    let end_soc =
        energy_model_ops::update_soc_percent(&soc, (&energy, &energy_unit), battery_capacity)?;

    state_model.add_energy(
        state,
        BevEnergyModel::TRIP_ENERGY_ELECTRIC,
        &energy,
        &energy_unit,
    )?;
    state_model.set_energy(
        state,
        BevEnergyModel::EDGE_ENERGY_ELECTRIC,
        &energy,
        &energy_unit,
    )?;
    state_model.set_custom_f64(state, BevEnergyModel::TRIP_SOC, &end_soc)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::prediction::{ModelType, PredictionModelConfig};
    use itertools::Itertools;
    use routee_compass_core::{model::unit::*, test::mock::traversal_model::TestTraversalModel};
    use std::path::PathBuf;

    #[test]
    fn test_bev_energy_model() {
        let (bat_cap, bat_unit) = (Energy::from(60.0), EnergyUnit::KilowattHours);
        let record = mock_prediction_model();
        let model = mock_traversal_model(record.clone(), 100.0, (&bat_cap, &bat_unit));
        let state_model = state_model(model);

        // starting at 100% SOC, we should be able to traverse a flat 110 miles at 60 mph
        // and it should use about half of the battery since the EPA range is 238 miles
        let distance = (Distance::from(110.0), DistanceUnit::Miles);
        let speed = (Speed::from(60.0), SpeedUnit::MPH);
        let grade = (Grade::from(0.0), GradeUnit::Decimal);
        let mut state = state_vector(&state_model, distance, speed, grade);

        bev_traversal(
            &mut state,
            &state_model,
            record.clone(),
            (&bat_cap, &bat_unit),
        )
        .unwrap();

        let (elec, _) = state_model
            .get_energy(&state, BevEnergyModel::EDGE_ENERGY_ELECTRIC, None)
            .expect("test invariant failed");

        assert!(elec.as_f64() > 0.0, "elec energy {} should be > 0.0", elec);

        let soc = state_model
            .get_custom_f64(&state, BevEnergyModel::TRIP_SOC)
            .unwrap();

        assert!(soc < 60.0, "soc {} should be < 60.0%", soc);
        assert!(soc > 40.0, "soc {} should be > 40.0%", soc);
    }

    #[test]
    fn test_bev_energy_model_regen() {
        let (bat_cap, bat_unit) = (Energy::from(60.0), EnergyUnit::KilowattHours);
        let record = mock_prediction_model();
        let model = mock_traversal_model(record.clone(), 20.0, (&bat_cap, &bat_unit));
        let state_model = state_model(model);

        // starting at 20% SOC, going downhill at -5% grade for 10 miles at 55mph, we should be see
        // some regen braking events and should end up with more energy than we started with
        let distance = (Distance::from(10.0), DistanceUnit::Miles);
        let speed = (Speed::from(55.0), SpeedUnit::MPH);
        let grade = (Grade::from(-5.0), GradeUnit::Percent);
        let mut state = state_vector(&state_model, distance, speed, grade);

        bev_traversal(
            &mut state,
            &state_model,
            record.clone(),
            (&bat_cap, &bat_unit),
        )
        .unwrap();

        let (elec, _) = state_model
            .get_energy(&state, BevEnergyModel::EDGE_ENERGY_ELECTRIC, None)
            .expect("test invariant failed");
        assert!(
            elec.as_f64() < 0.0,
            "elec energy {} should be < 0 (regen)",
            elec
        );

        let soc = state_model
            .get_custom_f64(&state, BevEnergyModel::TRIP_SOC)
            .unwrap();
        assert!(soc > 20.0, "soc {} should be > 20.0", soc);
        assert!(soc < 30.0, "soc {} should be < 30.0", soc);
    }

    #[test]
    fn test_bev_battery_in_bounds_upper() {
        // starting at 100% SOC, even going downhill with regen, we shouldn't be able to exceed 100%
        let (bat_cap, bat_unit) = (Energy::from(60.0), EnergyUnit::KilowattHours);
        let record = mock_prediction_model();
        let model = mock_traversal_model(record.clone(), 100.0, (&bat_cap, &bat_unit));
        let state_model = state_model(model);

        let distance = (Distance::from(10.0), DistanceUnit::Miles);
        let speed = (Speed::from(55.0), SpeedUnit::MPH);
        let grade = (Grade::from(-5.0), GradeUnit::Percent);
        let mut state = state_vector(&state_model, distance, speed, grade);

        bev_traversal(
            &mut state,
            &state_model,
            record.clone(),
            (&bat_cap, &bat_unit),
        )
        .unwrap();

        let battery_percent_soc = state_model
            .get_custom_f64(&state, BevEnergyModel::TRIP_SOC)
            .unwrap();
        assert!(battery_percent_soc <= 100.0);
    }

    #[test]
    fn test_bev_battery_in_bounds_lower() {
        // starting at 1% SOC, even going uphill, we shouldn't be able to go below 0%
        let (bat_cap, bat_unit) = (Energy::from(60.0), EnergyUnit::KilowattHours);
        let record = mock_prediction_model();
        let model = mock_traversal_model(record.clone(), 1.0, (&bat_cap, &bat_unit));
        let state_model = state_model(model);

        let distance = (Distance::from(100.0), DistanceUnit::Miles);
        let speed = (Speed::from(55.0), SpeedUnit::MPH);
        let grade = (Grade::from(5.0), GradeUnit::Percent);
        let mut state = state_vector(&state_model, distance, speed, grade);

        bev_traversal(
            &mut state,
            &state_model,
            record.clone(),
            (&bat_cap, &bat_unit),
        )
        .unwrap();

        let battery_percent_soc = state_model
            .get_custom_f64(&state, BevEnergyModel::TRIP_SOC)
            .unwrap();
        assert!(battery_percent_soc >= 0.0);
    }

    fn mock_prediction_model() -> Arc<PredictionModelRecord> {
        // let bat_cap = *battery_capacity.0;
        // let bat_unit = *battery_capacity.1;
        let model_file_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join("model")
            .join("test")
            .join("2017_CHEVROLET_Bolt.bin");
        let model_filename = model_file_path.to_str().expect("test invariant failed");

        let model_config = PredictionModelConfig::new(
            "Chevy Bolt".to_string(),
            model_filename.to_string(),
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
        );
        let model_record =
            PredictionModelRecord::try_from(&model_config).expect("test invariant failed");
        Arc::new(model_record)
    }

    fn mock_traversal_model(
        prediction_model_record: Arc<PredictionModelRecord>,
        starting_soc_percent: f64,
        battery_capacity: (&Energy, &EnergyUnit),
    ) -> Arc<dyn TraversalModel> {
        let bat_cap = *battery_capacity.0;
        let bat_unit = *battery_capacity.1;
        let staring_battery_energy: Energy =
            Energy::from(bat_cap.as_f64() * (starting_soc_percent * 0.01));

        let bev = BevEnergyModel::new(
            prediction_model_record,
            (bat_cap, bat_unit),
            (staring_battery_energy, bat_unit),
        )
        .expect("test invariant failed");

        // mock the upstream models via TestTraversalModel

        (TestTraversalModel::new(Arc::new(bev)).expect("test invariant failed")) as _
    }

    fn state_model(m: Arc<dyn TraversalModel>) -> StateModel {
        let out_f = m.output_features().into_iter().map(|(n, _)| n).join(", ");
        println!("output features: [{}]", out_f);
        let state_model = StateModel::empty()
            .register(m.input_features(), m.output_features())
            .expect("test invariant failed");
        println!(
            "registered state model features: {:?}",
            state_model.to_vec()
        );
        state_model
    }

    fn state_vector(
        state_model: &StateModel,
        distance: (Distance, DistanceUnit),
        speed: (Speed, SpeedUnit),
        grade: (Grade, GradeUnit),
    ) -> Vec<StateVariable> {
        let mut state = state_model.initial_state().unwrap();
        state_model
            .set_distance(
                &mut state,
                BevEnergyModel::EDGE_DISTANCE,
                &distance.0,
                &distance.1,
            )
            .expect("test invariant failed");
        state_model
            .set_speed(&mut state, BevEnergyModel::EDGE_SPEED, &speed.0, &speed.1)
            .expect("test invariant failed");
        state_model
            .set_grade(&mut state, BevEnergyModel::EDGE_GRADE, &grade.0, &grade.1)
            .expect("test invariant failed");
        state
    }
}
