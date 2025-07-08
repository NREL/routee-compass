use super::{
    energy_model_ops,
    prediction::{PredictionModelConfig, PredictionModelRecord},
};
use crate::model::fieldname;
use routee_compass_core::model::{
    network::{Edge, Vertex},
    state::{InputFeature, StateFeature, StateModel, StateVariable},
    traversal::{TraversalModel, TraversalModelError, TraversalModelService},
    unit::{EnergyRateUnit, EnergyUnit, RatioUnit, TimeUnit},
};
use serde_json::Value;
use std::sync::Arc;
use uom::{
    si::f64::{Energy, Ratio, Time},
    ConstZero,
};

#[derive(Clone)]
pub struct BevEnergyModel {
    prediction_model_record: Arc<PredictionModelRecord>,
    battery_capacity: Energy,
    starting_soc: Ratio,
}

impl BevEnergyModel {
    pub fn new(
        prediction_model_record: Arc<PredictionModelRecord>,
        battery_capacity: Energy,
        starting_battery_energy: Energy,
    ) -> Result<Self, TraversalModelError> {
        let starting_soc = energy_model_ops::soc_from_energy(
            starting_battery_energy,
            battery_capacity,
        )
        .map_err(|e| {
            TraversalModelError::BuildError(format!("Error building BEV Energy model due to {}", e))
        })?;
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
        match energy_model_ops::get_query_start_energy(query, self.battery_capacity)? {
            None => Ok(Arc::new(self.clone())),
            Some(starting_energy) => {
                let updated = Self::new(
                    self.prediction_model_record.clone(),
                    self.battery_capacity,
                    starting_energy,
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
        let battery_capacity = serde_json::from_value::<f64>(battery_capacity_conf.clone())
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
            battery_energy_unit.to_uom(battery_capacity),
            battery_energy_unit.to_uom(battery_capacity),
        )?;
        Ok(bev)
    }
}

impl TraversalModel for BevEnergyModel {
    fn name(&self) -> String {
        format!("BEV Energy Model: {}", self.prediction_model_record.name)
    }
    fn input_features(&self) -> Vec<InputFeature> {
        let mut input_features = vec![InputFeature::Distance {
            name: String::from(fieldname::EDGE_DISTANCE),
            unit: None,
        }];
        input_features.extend(self.prediction_model_record.input_features.clone());
        input_features
    }

    fn output_features(&self) -> Vec<(String, StateFeature)> {
        vec![
            (
                String::from(fieldname::TRIP_ENERGY),
                StateFeature::Energy {
                    value: Energy::ZERO,
                    accumulator: true,
                    output_unit: Some(EnergyUnit::KilowattHours),
                },
            ),
            (
                String::from(fieldname::TRIP_TIME),
                StateFeature::Time {
                    value: Time::ZERO,
                    accumulator: true,
                    output_unit: Some(TimeUnit::default()),
                },
            ),
            (
                String::from(fieldname::EDGE_TIME),
                StateFeature::Time {
                    value: Time::ZERO,
                    accumulator: false,
                    output_unit: Some(TimeUnit::default()),
                },
            ),
            (
                String::from(fieldname::EDGE_ENERGY),
                StateFeature::Energy {
                    value: Energy::ZERO,
                    accumulator: false,
                    output_unit: Some(EnergyUnit::KilowattHours),
                },
            ),
            (
                String::from(fieldname::TRIP_SOC),
                StateFeature::Ratio {
                    value: self.starting_soc,
                    accumulator: true,
                    output_unit: Some(RatioUnit::default()),
                },
            ),
            (
                String::from(fieldname::BATTERY_CAPACITY),
                StateFeature::Energy {
                    value: self.battery_capacity,
                    accumulator: false,
                    output_unit: Some(EnergyUnit::KilowattHours),
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
            self.battery_capacity,
        )
    }

    fn estimate_traversal(
        &self,
        _od: (&Vertex, &Vertex),
        state: &mut Vec<StateVariable>,
        state_model: &StateModel,
    ) -> Result<(), TraversalModelError> {
        bev_traversal_estimate(
            state,
            state_model,
            self.prediction_model_record.clone(),
            self.battery_capacity,
        )
    }
}

fn bev_traversal_estimate(
    state: &mut [StateVariable],
    state_model: &StateModel,
    record: Arc<PredictionModelRecord>,
    battery_capacity: Energy,
) -> Result<(), TraversalModelError> {
    // gather state variables
    let distance = state_model.get_distance(state, fieldname::EDGE_DISTANCE)?;
    let start_soc = state_model.get_ratio(state, fieldname::TRIP_SOC)?;

    let energy = match record.energy_rate_unit {
        EnergyRateUnit::KWHPM => {
            let distance_miles = distance.get::<uom::si::length::mile>();
            let energy_kwh = record.ideal_energy_rate * distance_miles;
            Energy::new::<uom::si::energy::kilowatt_hour>(energy_kwh)
        }
        EnergyRateUnit::KWHPKM => {
            let distance_km = distance.get::<uom::si::length::kilometer>();
            let energy_kwh = record.ideal_energy_rate * distance_km;
            Energy::new::<uom::si::energy::kilowatt_hour>(energy_kwh)
        }
        _ => {
            return Err(TraversalModelError::BuildError(format!(
                "unsupported energy rate unit: {}",
                record.energy_rate_unit
            )));
        }
    };
    let end_soc = energy_model_ops::update_soc_percent(start_soc, energy, battery_capacity)?;

    state_model.add_energy(state, fieldname::TRIP_ENERGY, &energy)?;
    state_model.set_energy(state, fieldname::EDGE_ENERGY, &energy)?;
    state_model.set_ratio(state, fieldname::TRIP_SOC, &end_soc)?;
    Ok(())
}

fn bev_traversal(
    state: &mut [StateVariable],
    state_model: &StateModel,
    record: Arc<PredictionModelRecord>,
    battery_capacity: Energy,
) -> Result<(), TraversalModelError> {
    // gather state variables
    let start_soc = state_model.get_ratio(state, fieldname::TRIP_SOC)?;

    // generate energy for link traversal
    let energy = record.predict(state, state_model)?;

    state_model.add_energy(state, fieldname::TRIP_ENERGY, &energy)?;
    state_model.set_energy(state, fieldname::EDGE_ENERGY, &energy)?;

    let end_soc = energy_model_ops::update_soc_percent(start_soc, energy, battery_capacity)?;

    state_model.set_ratio(state, fieldname::TRIP_SOC, &end_soc)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::prediction::{
        interpolation::feature_bounds::FeatureBounds, ModelType, PredictionModelConfig,
    };
    use routee_compass_core::{model::unit::*, testing::mock::traversal_model::TestTraversalModel};
    use std::{collections::HashMap, path::PathBuf};
    use uom::si::f64::{Length, Velocity};

    #[test]
    fn test_bev_energy_model() {
        let bat_cap = Energy::new::<uom::si::energy::kilowatt_hour>(60.0);
        let record = mock_prediction_model();
        let start_soc = Ratio::new::<uom::si::ratio::percent>(100.0);
        let model = mock_traversal_model(record.clone(), start_soc, bat_cap);
        let state_model = state_model(model);

        // starting at 100% SOC, we should be able to traverse a flat 110 miles at 60 mph
        // and it should use about half of the battery since the EPA range is 238 miles
        let distance = Length::new::<uom::si::length::mile>(110.0);
        let speed = Velocity::new::<uom::si::velocity::mile_per_hour>(60.0);
        let grade = Ratio::new::<uom::si::ratio::percent>(0.0);
        let mut state = state_vector(&state_model, distance, speed, grade);

        bev_traversal(&mut state, &state_model, record.clone(), bat_cap).unwrap();

        let elec = state_model
            .get_energy(&state, fieldname::TRIP_ENERGY)
            .expect("test invariant failed");

        assert!(
            elec > Energy::ZERO,
            "elec energy {:?} should be > 0.0",
            elec
        );

        let soc = state_model.get_ratio(&state, fieldname::TRIP_SOC).unwrap();
        let lower_bound = Ratio::new::<uom::si::ratio::percent>(40.0);
        let upper_bound = Ratio::new::<uom::si::ratio::percent>(60.0);

        assert!(soc < upper_bound, "soc {:?} should be < 60.0%", soc);
        assert!(soc > lower_bound, "soc {:?} should be > 40.0%", soc);
    }

    #[test]
    fn test_bev_energy_model_regen() {
        let bat_cap = Energy::new::<uom::si::energy::kilowatt_hour>(60.0);
        let record = mock_prediction_model();
        let start_soc = Ratio::new::<uom::si::ratio::percent>(20.0);
        let model = mock_traversal_model(record.clone(), start_soc, bat_cap);
        let state_model = state_model(model);

        // starting at 20% SOC, going downhill at -5% grade for 10 miles at 55mph, we should be see
        // some regen braking events and should end up with more energy than we started with
        let distance = Length::new::<uom::si::length::mile>(10.0);
        let speed = Velocity::new::<uom::si::velocity::mile_per_hour>(55.0);
        let grade = Ratio::new::<uom::si::ratio::percent>(-5.0);
        let mut state = state_vector(&state_model, distance, speed, grade);

        bev_traversal(&mut state, &state_model, record.clone(), bat_cap).unwrap();

        let elec = state_model
            .get_energy(&state, fieldname::TRIP_ENERGY)
            .expect("test invariant failed");
        assert!(
            elec < Energy::ZERO,
            "elec energy {:?} should be < 0 (regen)",
            elec
        );

        let soc = state_model.get_ratio(&state, fieldname::TRIP_SOC).unwrap();
        let lower_bound = Ratio::new::<uom::si::ratio::percent>(20.0);
        let upper_bound = Ratio::new::<uom::si::ratio::percent>(30.0);
        assert!(soc < upper_bound, "soc {:?} should be < 30.0%", soc);
        assert!(soc > lower_bound, "soc {:?} should be > 20.0%", soc);
    }

    #[test]
    fn test_bev_battery_in_bounds_upper() {
        // starting at 100% SOC, even going downhill with regen, we shouldn't be able to exceed 100%
        let bat_cap = Energy::new::<uom::si::energy::kilowatt_hour>(60.0);
        let record = mock_prediction_model();
        let start_soc = Ratio::new::<uom::si::ratio::percent>(100.0);
        let model = mock_traversal_model(record.clone(), start_soc, bat_cap);
        let state_model = state_model(model);

        let distance = Length::new::<uom::si::length::mile>(10.0);
        let speed = Velocity::new::<uom::si::velocity::mile_per_hour>(55.0);
        let grade = Ratio::new::<uom::si::ratio::percent>(-5.0);
        let mut state = state_vector(&state_model, distance, speed, grade);

        bev_traversal(&mut state, &state_model, record.clone(), bat_cap).unwrap();

        let battery_percent_soc = state_model.get_ratio(&state, fieldname::TRIP_SOC).unwrap();
        assert!(battery_percent_soc <= Ratio::new::<uom::si::ratio::percent>(100.0));
    }

    #[test]
    fn test_bev_battery_in_bounds_lower() {
        // starting at 1% SOC, even going uphill, we shouldn't be able to go below 0%
        let bat_cap = Energy::new::<uom::si::energy::kilowatt_hour>(60.0);
        let record = mock_prediction_model();
        let start_soc = Ratio::new::<uom::si::ratio::percent>(100.0);
        let model = mock_traversal_model(record.clone(), start_soc, bat_cap);
        let state_model = state_model(model);

        let distance = Length::new::<uom::si::length::mile>(100.0);
        let speed = Velocity::new::<uom::si::velocity::mile_per_hour>(55.0);
        let grade = Ratio::new::<uom::si::ratio::percent>(5.0);
        let mut state = state_vector(&state_model, distance, speed, grade);

        bev_traversal(&mut state, &state_model, record.clone(), bat_cap).unwrap();

        let battery_percent_soc = state_model.get_ratio(&state, fieldname::TRIP_SOC).unwrap();
        assert!(battery_percent_soc >= Ratio::ZERO);
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

        let feature_bounds = HashMap::from([
            (
                fieldname::EDGE_SPEED.to_string(),
                FeatureBounds {
                    lower_bound: 0.0,
                    upper_bound: 100.0,
                    num_bins: 101,
                },
            ),
            (
                fieldname::EDGE_GRADE.to_string(),
                FeatureBounds {
                    lower_bound: -0.2,
                    upper_bound: 0.2,
                    num_bins: 41,
                },
            ),
        ]);

        let input_features = vec![
            InputFeature::Speed {
                name: fieldname::EDGE_SPEED.to_string(),
                unit: Some(SpeedUnit::MPH),
            },
            InputFeature::Ratio {
                name: fieldname::EDGE_GRADE.to_string(),
                unit: Some(RatioUnit::Decimal),
            },
        ];

        let model_config = PredictionModelConfig::new(
            "Chevy Bolt".to_string(),
            model_filename.to_string(),
            ModelType::Interpolate {
                underlying_model_type: Box::new(ModelType::Smartcore),
                feature_bounds,
            },
            input_features,
            EnergyRateUnit::KWHPM,
            Some(1.3958),
        );
        let model_record =
            PredictionModelRecord::try_from(&model_config).expect("test invariant failed");
        Arc::new(model_record)
    }

    fn mock_traversal_model(
        prediction_model_record: Arc<PredictionModelRecord>,
        starting_soc: Ratio,
        battery_capacity: Energy,
    ) -> Arc<dyn TraversalModel> {
        let starting_energy = battery_capacity * starting_soc;
        let bev = BevEnergyModel::new(prediction_model_record, battery_capacity, starting_energy)
            .expect("test invariant failed");

        // mock the upstream models via TestTraversalModel

        (TestTraversalModel::new(Arc::new(bev)).expect("test invariant failed")) as _
    }

    fn state_model(m: Arc<dyn TraversalModel>) -> StateModel {
        StateModel::empty()
            .register(m.input_features(), m.output_features())
            .expect("test invariant failed")
    }

    fn state_vector(
        state_model: &StateModel,
        distance: Length,
        speed: Velocity,
        grade: Ratio,
    ) -> Vec<StateVariable> {
        let mut state = state_model.initial_state().unwrap();
        state_model
            .set_distance(&mut state, fieldname::EDGE_DISTANCE, &distance)
            .expect("test invariant failed");
        state_model
            .set_speed(&mut state, fieldname::EDGE_SPEED, &speed)
            .expect("test invariant failed");
        state_model
            .set_ratio(&mut state, fieldname::EDGE_GRADE, &grade)
            .expect("test invariant failed");
        state
    }
}
