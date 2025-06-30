use super::{
    energy_model_ops,
    prediction::{PredictionModelConfig, PredictionModelRecord},
};
use crate::model::{charging_station_locator::ChargingStationLocator, fieldname};
use routee_compass_core::{
    config::ConfigJsonExtensions,
    model::{
        network::{Edge, Vertex, VertexId},
        state::{InputFeature, StateFeature, StateModel, StateVariable},
        traversal::{TraversalModel, TraversalModelError, TraversalModelService},
        unit::{EnergyRateUnit, EnergyUnit, RatioUnit, TimeUnit},
    },
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
    charging_station_locator: Arc<ChargingStationLocator>,
}

impl BevEnergyModel {
    pub fn new(
        prediction_model_record: Arc<PredictionModelRecord>,
        battery_capacity: Energy,
        starting_battery_energy: Energy,
        charging_station_locator: Arc<ChargingStationLocator>,
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
            charging_station_locator,
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
                    self.charging_station_locator.clone(),
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

        let charging_station_input_file = value
            .get_config_path_optional(&"charging_station_input_file", &"bev_energy_model")
            .map_err(|e| {
                TraversalModelError::BuildError(format!(
                    "failure reading 'charging_station_input_file' from bev energy model configuration: {}",
                    e
                ))
            })?;

        let charging_station_locator = match charging_station_input_file {
            Some(file) => {
                let locator = ChargingStationLocator::from_csv_file(&file).map_err(|e| {
                    TraversalModelError::BuildError(format!(
                        "failed to load charging station locator: {}",
                        e
                    ))
                })?;
                Arc::new(locator)
            }
            None => Arc::new(ChargingStationLocator::default()),
        };

        let bev = BevEnergyModel::new(
            Arc::new(prediction_model),
            battery_energy_unit.to_uom(battery_capacity),
            battery_energy_unit.to_uom(battery_capacity),
            charging_station_locator,
        )?;
        Ok(bev)
    }
}

impl TraversalModel for BevEnergyModel {
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
                    accumulator: false,
                    output_unit: Some(RatioUnit::default()),
                },
            ),
        ]
    }

    fn traverse_edge(
        &self,
        trajectory: (&Vertex, &Edge, &Vertex),
        state: &mut Vec<StateVariable>,
        state_model: &StateModel,
    ) -> Result<(), TraversalModelError> {
        bev_traversal(
            state,
            state_model,
            self.prediction_model_record.clone(),
            self.battery_capacity,
            &trajectory.2.vertex_id,
            self.charging_station_locator.clone(),
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
    destination_vertex_id: &VertexId,
    charging_station_locator: Arc<ChargingStationLocator>,
) -> Result<(), TraversalModelError> {
    // gather state variables
    let start_soc = state_model.get_ratio(state, fieldname::TRIP_SOC)?;

    // generate energy for link traversal
    let energy = record.predict(state, state_model)?;

    state_model.add_energy(state, fieldname::TRIP_ENERGY, &energy)?;
    state_model.set_energy(state, fieldname::EDGE_ENERGY, &energy)?;

    let end_soc = energy_model_ops::update_soc_percent(start_soc, energy, battery_capacity)?;

    // check if we are at a charging station
    if let Some(charging_station) = charging_station_locator.get_station(destination_vertex_id) {
        // if we are at a charging station, we can charge the battery to full
        let full_soc = Ratio::new::<uom::si::ratio::percent>(100.0);
        let soc_to_full = full_soc - end_soc;
        let charge_energy = soc_to_full * battery_capacity;
        let time_to_charge: Time = charge_energy / charging_station.power();

        state_model.set_ratio(state, fieldname::TRIP_SOC, &full_soc)?;
        state_model.add_time(state, fieldname::TRIP_TIME, &time_to_charge)?;
    } else {
        state_model.set_ratio(state, fieldname::TRIP_SOC, &end_soc)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{
        charging_station_locator::ChargingStation,
        prediction::{
            interpolation::feature_bounds::FeatureBounds, ModelType, PredictionModelConfig,
        },
    };
    use routee_compass_core::{
        model::{network::VertexId, unit::*},
        testing::mock::traversal_model::TestTraversalModel,
    };
    use std::{collections::HashMap, path::PathBuf};
    use uom::si::f64::{Length, Power, Velocity};

    #[test]
    fn test_bev_energy_model() {
        let bat_cap = Energy::new::<uom::si::energy::kilowatt_hour>(60.0);
        let record = mock_prediction_model();
        let start_soc = Ratio::new::<uom::si::ratio::percent>(100.0);
        let charging_station_locator = mock_charging_station_locator();
        // we don't want charging so don't use vertex 1
        let destination_vertex_id = VertexId(99);
        let model = mock_traversal_model(
            record.clone(),
            start_soc,
            bat_cap,
            charging_station_locator.clone(),
        );
        let state_model = state_model(model);

        // starting at 100% SOC, we should be able to traverse a flat 110 miles at 60 mph
        // and it should use about half of the battery since the EPA range is 238 miles
        let distance = Length::new::<uom::si::length::mile>(110.0);
        let speed = Velocity::new::<uom::si::velocity::mile_per_hour>(60.0);
        let grade = Ratio::new::<uom::si::ratio::percent>(0.0);
        let mut state = state_vector(&state_model, distance, speed, grade);

        bev_traversal(
            &mut state,
            &state_model,
            record.clone(),
            bat_cap,
            &destination_vertex_id,
            charging_station_locator.clone(),
        )
        .unwrap();

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
        // we don't want charging so don't use vertex 1
        let destination_vertex_id = VertexId(99);
        let charging_station_locator = mock_charging_station_locator();
        let model = mock_traversal_model(
            record.clone(),
            start_soc,
            bat_cap,
            charging_station_locator.clone(),
        );
        let state_model = state_model(model);

        // starting at 20% SOC, going downhill at -5% grade for 10 miles at 55mph, we should be see
        // some regen braking events and should end up with more energy than we started with
        let distance = Length::new::<uom::si::length::mile>(10.0);
        let speed = Velocity::new::<uom::si::velocity::mile_per_hour>(55.0);
        let grade = Ratio::new::<uom::si::ratio::percent>(-5.0);
        let mut state = state_vector(&state_model, distance, speed, grade);

        bev_traversal(
            &mut state,
            &state_model,
            record.clone(),
            bat_cap,
            &destination_vertex_id,
            charging_station_locator.clone(),
        )
        .unwrap();

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
        let destination_vertex_id = VertexId(1);
        let charging_station_locator = mock_charging_station_locator();
        let model = mock_traversal_model(
            record.clone(),
            start_soc,
            bat_cap,
            charging_station_locator.clone(),
        );
        let state_model = state_model(model);

        let distance = Length::new::<uom::si::length::mile>(10.0);
        let speed = Velocity::new::<uom::si::velocity::mile_per_hour>(55.0);
        let grade = Ratio::new::<uom::si::ratio::percent>(-5.0);
        let mut state = state_vector(&state_model, distance, speed, grade);

        bev_traversal(
            &mut state,
            &state_model,
            record.clone(),
            bat_cap,
            &destination_vertex_id,
            charging_station_locator.clone(),
        )
        .unwrap();

        let battery_percent_soc = state_model.get_ratio(&state, fieldname::TRIP_SOC).unwrap();
        assert!(battery_percent_soc <= Ratio::new::<uom::si::ratio::percent>(100.0));
    }

    #[test]
    fn test_bev_battery_in_bounds_lower() {
        // starting at 1% SOC, even going uphill, we shouldn't be able to go below 0%
        let bat_cap = Energy::new::<uom::si::energy::kilowatt_hour>(60.0);
        let record = mock_prediction_model();
        let start_soc = Ratio::new::<uom::si::ratio::percent>(100.0);
        let destination_vertex_id = VertexId(1);
        let charging_station_locator = mock_charging_station_locator();
        let model = mock_traversal_model(
            record.clone(),
            start_soc,
            bat_cap,
            charging_station_locator.clone(),
        );
        let state_model = state_model(model);

        let distance = Length::new::<uom::si::length::mile>(100.0);
        let speed = Velocity::new::<uom::si::velocity::mile_per_hour>(55.0);
        let grade = Ratio::new::<uom::si::ratio::percent>(5.0);
        let mut state = state_vector(&state_model, distance, speed, grade);

        bev_traversal(
            &mut state,
            &state_model,
            record.clone(),
            bat_cap,
            &destination_vertex_id,
            charging_station_locator.clone(),
        )
        .unwrap();

        let battery_percent_soc = state_model.get_ratio(&state, fieldname::TRIP_SOC).unwrap();
        assert!(battery_percent_soc >= Ratio::ZERO);
    }

    #[test]
    fn test_bev_charging_at_station_from_low_soc() {
        // Test charging behavior when arriving at a charging station with low SOC
        let bat_cap = Energy::new::<uom::si::energy::kilowatt_hour>(60.0);
        let record = mock_prediction_model();
        let start_soc = Ratio::new::<uom::si::ratio::percent>(15.0);
        let destination_vertex_id = VertexId(1); // This vertex has a charging station
        let charging_station_locator = mock_charging_station_locator();
        let model = mock_traversal_model(
            record.clone(),
            start_soc,
            bat_cap,
            charging_station_locator.clone(),
        );
        let state_model = state_model(model);

        // Travel a short distance to a charging station
        let distance = Length::new::<uom::si::length::mile>(5.0);
        let speed = Velocity::new::<uom::si::velocity::mile_per_hour>(30.0);
        let grade = Ratio::new::<uom::si::ratio::percent>(0.0);
        let mut state = state_vector(&state_model, distance, speed, grade);

        bev_traversal(
            &mut state,
            &state_model,
            record.clone(),
            bat_cap,
            &destination_vertex_id,
            charging_station_locator.clone(),
        )
        .unwrap();

        // After charging, SOC should be 100%
        let final_soc = state_model.get_ratio(&state, fieldname::TRIP_SOC).unwrap();
        let expected_soc = Ratio::new::<uom::si::ratio::percent>(100.0);
        assert_eq!(final_soc, expected_soc, "SOC should be 100% after charging");

        // Should have added charging time to trip time
        let trip_time = state_model.get_time(&state, fieldname::TRIP_TIME).unwrap();
        assert!(
            trip_time > Time::ZERO,
            "Should have added charging time to trip"
        );
    }

    #[test]
    fn test_bev_charging_at_station_from_medium_soc() {
        // Test charging behavior when arriving at a charging station with medium SOC
        let bat_cap = Energy::new::<uom::si::energy::kilowatt_hour>(60.0);
        let record = mock_prediction_model();
        let start_soc = Ratio::new::<uom::si::ratio::percent>(50.0);
        let destination_vertex_id = VertexId(1); // This vertex has a charging station
        let charging_station_locator = mock_charging_station_locator();
        let model = mock_traversal_model(
            record.clone(),
            start_soc,
            bat_cap,
            charging_station_locator.clone(),
        );
        let state_model = state_model(model);

        // Travel a short distance to a charging station
        let distance = Length::new::<uom::si::length::mile>(2.0);
        let speed = Velocity::new::<uom::si::velocity::mile_per_hour>(25.0);
        let grade = Ratio::new::<uom::si::ratio::percent>(0.0);
        let mut state = state_vector(&state_model, distance, speed, grade);

        bev_traversal(
            &mut state,
            &state_model,
            record.clone(),
            bat_cap,
            &destination_vertex_id,
            charging_station_locator.clone(),
        )
        .unwrap();

        // After charging, SOC should be 100%
        let final_soc = state_model.get_ratio(&state, fieldname::TRIP_SOC).unwrap();
        let expected_soc = Ratio::new::<uom::si::ratio::percent>(100.0);
        assert_eq!(final_soc, expected_soc, "SOC should be 100% after charging");

        // Should have added charging time (less than when starting from low SOC)
        let trip_time = state_model.get_time(&state, fieldname::TRIP_TIME).unwrap();
        assert!(
            trip_time > Time::ZERO,
            "Should have added charging time to trip"
        );
    }

    #[test]
    fn test_bev_no_charging_at_non_station_vertex() {
        // Test that no charging occurs when not at a charging station
        let bat_cap = Energy::new::<uom::si::energy::kilowatt_hour>(60.0);
        let record = mock_prediction_model();
        let start_soc = Ratio::new::<uom::si::ratio::percent>(30.0);
        let destination_vertex_id = VertexId(99); // This vertex has NO charging station
        let charging_station_locator = mock_charging_station_locator();
        let model = mock_traversal_model(
            record.clone(),
            start_soc,
            bat_cap,
            charging_station_locator.clone(),
        );
        let state_model = state_model(model);

        // Travel a distance that would normally consume some energy
        let distance = Length::new::<uom::si::length::mile>(10.0);
        let speed = Velocity::new::<uom::si::velocity::mile_per_hour>(45.0);
        let grade = Ratio::new::<uom::si::ratio::percent>(2.0);
        let mut state = state_vector(&state_model, distance, speed, grade);

        bev_traversal(
            &mut state,
            &state_model,
            record.clone(),
            bat_cap,
            &destination_vertex_id,
            charging_station_locator.clone(),
        )
        .unwrap();

        // SOC should be less than starting SOC (no charging occurred)
        let final_soc = state_model.get_ratio(&state, fieldname::TRIP_SOC).unwrap();
        assert!(
            final_soc < start_soc,
            "SOC should decrease when not at charging station"
        );

        // Trip time should only include travel time, no charging time
        let trip_time = state_model.get_time(&state, fieldname::TRIP_TIME).unwrap();
        assert_eq!(
            trip_time,
            Time::ZERO,
            "Should not add charging time when not at charging station"
        );
    }

    #[test]
    fn test_bev_charging_at_station_from_full_battery() {
        // Test charging behavior when arriving at a charging station with full battery
        let bat_cap = Energy::new::<uom::si::energy::kilowatt_hour>(60.0);
        let record = mock_prediction_model();
        let start_soc = Ratio::new::<uom::si::ratio::percent>(100.0);
        let destination_vertex_id = VertexId(1); // This vertex has a charging station
        let charging_station_locator = mock_charging_station_locator();
        let model = mock_traversal_model(
            record.clone(),
            start_soc,
            bat_cap,
            charging_station_locator.clone(),
        );
        let state_model = state_model(model);

        // Travel a very short distance that uses minimal energy
        let distance = Length::new::<uom::si::length::mile>(0.5);
        let speed = Velocity::new::<uom::si::velocity::mile_per_hour>(15.0);
        let grade = Ratio::new::<uom::si::ratio::percent>(0.0);
        let mut state = state_vector(&state_model, distance, speed, grade);

        bev_traversal(
            &mut state,
            &state_model,
            record.clone(),
            bat_cap,
            &destination_vertex_id,
            charging_station_locator.clone(),
        )
        .unwrap();

        // After "charging", SOC should still be 100%
        let final_soc = state_model.get_ratio(&state, fieldname::TRIP_SOC).unwrap();
        let expected_soc = Ratio::new::<uom::si::ratio::percent>(100.0);
        assert_eq!(
            final_soc, expected_soc,
            "SOC should remain 100% when charging from nearly full"
        );

        // Should have minimal or no charging time since battery was nearly full
        let trip_time = state_model.get_time(&state, fieldname::TRIP_TIME).unwrap();
        // Time should be very small since we're charging from ~99% to 100%
        assert!(
            trip_time >= Time::ZERO,
            "Charging time should be non-negative"
        );
    }

    #[test]
    fn test_bev_charging_time_calculation() {
        // Test that charging time is calculated correctly based on SOC difference and charging power
        let bat_cap = Energy::new::<uom::si::energy::kilowatt_hour>(60.0);
        let record = mock_prediction_model();
        let start_soc = Ratio::new::<uom::si::ratio::percent>(20.0);
        let destination_vertex_id = VertexId(1); // This vertex has a 50kW charging station
        let charging_station_locator = mock_charging_station_locator();
        let model = mock_traversal_model(
            record.clone(),
            start_soc,
            bat_cap,
            charging_station_locator.clone(),
        );
        let state_model = state_model(model);

        // Travel a short distance that uses some energy
        let distance = Length::new::<uom::si::length::mile>(5.0);
        let speed = Velocity::new::<uom::si::velocity::mile_per_hour>(30.0);
        let grade = Ratio::new::<uom::si::ratio::percent>(1.0);
        let mut state = state_vector(&state_model, distance, speed, grade);

        bev_traversal(
            &mut state,
            &state_model,
            record.clone(),
            bat_cap,
            &destination_vertex_id,
            charging_station_locator.clone(),
        )
        .unwrap();

        // Calculate expected charging time
        // Starting from ~15-20% SOC (after consuming some energy), charging to 100%
        let final_soc = state_model.get_ratio(&state, fieldname::TRIP_SOC).unwrap();
        assert_eq!(final_soc, Ratio::new::<uom::si::ratio::percent>(100.0));

        let trip_time = state_model.get_time(&state, fieldname::TRIP_TIME).unwrap();

        // With a 60kWh battery and 50kW charging power, charging from ~15% to 100% should take
        // approximately (85% * 60kWh) / 50kW = ~1.02 hours
        let expected_min_time = Time::new::<uom::si::time::hour>(0.8); // At least 48 minutes
        let expected_max_time = Time::new::<uom::si::time::hour>(1.3); // At most 78 minutes

        assert!(
            trip_time >= expected_min_time,
            "Charging time {:?} should be at least {:?}",
            trip_time,
            expected_min_time
        );
        assert!(
            trip_time <= expected_max_time,
            "Charging time {:?} should be at most {:?}",
            trip_time,
            expected_max_time
        );
    }

    fn mock_charging_station_locator() -> Arc<ChargingStationLocator> {
        let mut station_map = HashMap::new();
        // Mock a charging station at vertex 1 with 50 kW power and $0.20 per kWh
        station_map.insert(
            VertexId(1),
            ChargingStation::L2 {
                power: Power::new::<uom::si::power::kilowatt>(50.0),
                cost_per_kwh: 0.20,
            },
        );
        Arc::new(ChargingStationLocator::new(station_map))
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
        charging_station_locator: Arc<ChargingStationLocator>,
    ) -> Arc<dyn TraversalModel> {
        let starting_energy = battery_capacity * starting_soc;
        let bev = BevEnergyModel::new(
            prediction_model_record,
            battery_capacity,
            starting_energy,
            charging_station_locator,
        )
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
