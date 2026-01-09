use std::{collections::HashSet, sync::Arc};

use routee_compass_core::{
    algorithm::search::SearchTree,
    model::{
        network::{Edge, Vertex},
        state::{StateModel, StateVariable},
        traversal::{TraversalModel, TraversalModelError},
    },
};
use uom::si::f64::{Ratio, Time};

use crate::model::{
    charging::charging_station_locator::{ChargingStationLocator, PowerType},
    fieldname,
};

pub struct SimpleChargingModel {
    pub charging_station_locator: Arc<ChargingStationLocator>,
    pub starting_soc: Ratio,
    pub full_soc: Ratio,
    pub charge_soc_threshold: Ratio,
    pub valid_power_types: HashSet<PowerType>,
}

impl TraversalModel for SimpleChargingModel {
    fn name(&self) -> String {
        "Simple Charging Model".to_string()
    }

    fn estimate_traversal(
        &self,
        _od: (&Vertex, &Vertex),
        _state: &mut Vec<StateVariable>,
        _tree: &SearchTree,
        _state_model: &StateModel,
    ) -> Result<(), TraversalModelError> {
        Ok(())
    }
    fn traverse_edge(
        &self,
        trajectory: (&Vertex, &Edge, &Vertex),
        state: &mut Vec<StateVariable>,
        _tree: &SearchTree,
        state_model: &StateModel,
    ) -> Result<(), TraversalModelError> {
        let current_soc = state_model.get_ratio(state, fieldname::TRIP_SOC)?;
        let battery_capacity = state_model.get_energy(state, fieldname::BATTERY_CAPACITY)?;
        let (_start_vertex, _edge, end_vertex) = trajectory;
        if let Some(charging_station) = self
            .charging_station_locator
            .get_station(&end_vertex.vertex_id)
        {
            let should_charge = current_soc < self.charge_soc_threshold
                && self
                    .valid_power_types
                    .contains(&charging_station.power_type);
            if should_charge {
                let soc_to_full = self.full_soc - current_soc;
                let charge_energy = soc_to_full * battery_capacity;
                let time_to_charge: Time = charge_energy / charging_station.power;

                state_model.set_ratio(state, fieldname::TRIP_SOC, &self.full_soc)?;
                state_model.add_time(state, fieldname::TRIP_TIME, &time_to_charge)?;
                state_model.add_time(state, fieldname::EDGE_TIME, &time_to_charge)?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{
        charging::{
            charging_station_locator::{ChargingStation, ChargingStationLocator, PowerType},
            simple_charging_service::SimpleChargingService,
        },
        fieldname,
    };
    use geo::coord;
    use routee_compass_core::{
        model::{
            network::{Edge, EdgeId, EdgeListId, Vertex, VertexId},
            state::{InputFeature, StateModel, StateVariable, StateVariableConfig},
            traversal::TraversalModelService,
        },
        util::geo::InternalCoord,
    };
    use std::collections::HashMap;
    use uom::{
        si::f64::{Energy, Length, Power, Ratio, Time},
        ConstZero,
    };

    fn mock_charging_station_locator() -> Arc<ChargingStationLocator> {
        let mut stations = HashMap::new();

        // Add a DC fast charging station at vertex 1
        stations.insert(
            VertexId(1),
            ChargingStation {
                power: Power::new::<uom::si::power::kilowatt>(50.0), // 50 kW DC fast charger
                power_type: PowerType::DCFC,
                cost_per_kwh: 0.30,
            },
        );

        // Add a Level 2 AC charging station at vertex 2
        stations.insert(
            VertexId(2),
            ChargingStation {
                power: Power::new::<uom::si::power::kilowatt>(7.2), // 7.2 kW Level 2 charger
                power_type: PowerType::L2,
                cost_per_kwh: 0.15,
            },
        );

        Arc::new(ChargingStationLocator::new(stations))
    }

    fn mock_simple_charging_service() -> Arc<SimpleChargingService> {
        let charging_station_locator = mock_charging_station_locator();
        let starting_soc = Ratio::new::<uom::si::ratio::percent>(50.0);
        let full_soc = Ratio::new::<uom::si::ratio::percent>(100.0);
        let charge_soc_threshold = Ratio::new::<uom::si::ratio::percent>(20.0);
        let valid_power_types = vec![PowerType::DCFC, PowerType::L2].into_iter().collect();

        Arc::new(SimpleChargingService {
            charging_station_locator,
            starting_soc,
            full_soc,
            charge_soc_threshold,
            valid_power_types,
        })
    }

    fn state_model(service: Arc<dyn TraversalModelService>) -> StateModel {
        // Create a state model that includes all the required features
        let mut input_features = vec![
            InputFeature::Ratio {
                name: fieldname::TRIP_SOC.to_string(),
                unit: None,
            },
            InputFeature::Energy {
                name: fieldname::BATTERY_CAPACITY.to_string(),
                unit: None,
            },
        ];

        // Add features from the service
        input_features.extend(service.input_features());

        // Create output features - we need to provide battery_capacity as an output feature
        let mut output_features = vec![(
            fieldname::BATTERY_CAPACITY.to_string(),
            StateVariableConfig::Energy {
                initial: Energy::new::<uom::si::energy::kilowatt_hour>(60.0),
                accumulator: false,
                output_unit: None,
            },
        )];

        // Add features from the service
        output_features.extend(service.output_features());

        StateModel::empty()
            .register(input_features, output_features)
            .expect("test invariant failed")
    }

    fn state_vector(
        state_model: &StateModel,
        trip_soc: Ratio,
        battery_capacity: Energy,
    ) -> Vec<StateVariable> {
        let mut state = state_model.initial_state(None).unwrap();
        state_model
            .set_ratio(&mut state, fieldname::TRIP_SOC, &trip_soc)
            .expect("test invariant failed");
        state_model
            .set_energy(&mut state, fieldname::BATTERY_CAPACITY, &battery_capacity)
            .expect("test invariant failed");
        state
    }

    fn mock_trajectory(vertex_id: usize) -> (Vertex, Edge, Vertex) {
        let start_vertex = Vertex {
            vertex_id: VertexId(0),
            coordinate: InternalCoord(coord! {x: 0.0f32, y: 0.0f32}),
        };
        let edge = Edge {
            edge_list_id: EdgeListId(0),
            edge_id: EdgeId(0),
            src_vertex_id: VertexId(0),
            dst_vertex_id: VertexId(vertex_id),
            distance: Length::new::<uom::si::length::meter>(1000.0),
        };
        let end_vertex = Vertex {
            vertex_id: VertexId(vertex_id),
            coordinate: InternalCoord(coord! {x: 1.0f32, y: 1.0f32}),
        };
        (start_vertex, edge, end_vertex)
    }

    #[test]
    fn test_charging_when_soc_below_threshold() {
        let service = mock_simple_charging_service();
        let state_model = state_model(service);
        let tree = SearchTree::default();

        // Set SOC to 15% (below 20% threshold) and 60 kWh battery
        let low_soc = Ratio::new::<uom::si::ratio::percent>(15.0);
        let battery_capacity = Energy::new::<uom::si::energy::kilowatt_hour>(60.0);
        let mut state = state_vector(&state_model, low_soc, battery_capacity);

        // Traverse to vertex 1 (DC fast charging station)
        let trajectory = mock_trajectory(1);

        let charging_model = SimpleChargingModel {
            charging_station_locator: mock_charging_station_locator(),
            starting_soc: Ratio::new::<uom::si::ratio::percent>(50.0),
            full_soc: Ratio::new::<uom::si::ratio::percent>(100.0),
            charge_soc_threshold: Ratio::new::<uom::si::ratio::percent>(20.0),
            valid_power_types: vec![PowerType::DCFC, PowerType::L2].into_iter().collect(),
        };

        charging_model
            .traverse_edge(
                (&trajectory.0, &trajectory.1, &trajectory.2),
                &mut state,
                &tree,
                &state_model,
            )
            .unwrap();

        // Check that SOC was updated to full (100%)
        let final_soc = state_model.get_ratio(&state, fieldname::TRIP_SOC).unwrap();
        assert_eq!(final_soc, Ratio::new::<uom::si::ratio::percent>(100.0));

        // Check that charging time was added
        let trip_time = state_model.get_time(&state, fieldname::TRIP_TIME).unwrap();
        let edge_time = state_model.get_time(&state, fieldname::EDGE_TIME).unwrap();

        // Should take time to charge from 15% to 100% (85% of 60 kWh = 51 kWh at 50 kW = ~1.02 hours)
        let expected_charge_time = Time::new::<uom::si::time::hour>(51.0 / 50.0);
        assert!((trip_time - expected_charge_time).abs() < Time::new::<uom::si::time::second>(1.0));
        assert!((edge_time - expected_charge_time).abs() < Time::new::<uom::si::time::second>(1.0));
    }

    #[test]
    fn test_no_charging_when_soc_above_threshold() {
        let service = mock_simple_charging_service();
        let state_model = state_model(service);
        let tree = SearchTree::default();

        // Set SOC to 50% (above 20% threshold)
        let high_soc = Ratio::new::<uom::si::ratio::percent>(50.0);
        let battery_capacity = Energy::new::<uom::si::energy::kilowatt_hour>(60.0);
        let mut state = state_vector(&state_model, high_soc, battery_capacity);

        // Traverse to vertex 1 (DC fast charging station)
        let trajectory = mock_trajectory(1);

        let charging_model = SimpleChargingModel {
            charging_station_locator: mock_charging_station_locator(),
            starting_soc: Ratio::new::<uom::si::ratio::percent>(50.0),
            full_soc: Ratio::new::<uom::si::ratio::percent>(100.0),
            charge_soc_threshold: Ratio::new::<uom::si::ratio::percent>(20.0),
            valid_power_types: vec![PowerType::DCFC, PowerType::L2].into_iter().collect(),
        };

        charging_model
            .traverse_edge(
                (&trajectory.0, &trajectory.1, &trajectory.2),
                &mut state,
                &tree,
                &state_model,
            )
            .unwrap();

        // Check that SOC remained unchanged
        let final_soc = state_model.get_ratio(&state, fieldname::TRIP_SOC).unwrap();
        assert_eq!(final_soc, high_soc);

        // Check that no charging time was added
        let trip_time = state_model.get_time(&state, fieldname::TRIP_TIME).unwrap();
        let edge_time = state_model.get_time(&state, fieldname::EDGE_TIME).unwrap();
        assert_eq!(trip_time, Time::ZERO);
        assert_eq!(edge_time, Time::ZERO);
    }

    #[test]
    fn test_no_charging_station_at_vertex() {
        let service = mock_simple_charging_service();
        let state_model = state_model(service.clone());
        let tree = SearchTree::default();

        // Set SOC to 15% (below 20% threshold)
        let low_soc = Ratio::new::<uom::si::ratio::percent>(15.0);
        let battery_capacity = Energy::new::<uom::si::energy::kilowatt_hour>(60.0);
        let mut state = state_vector(&state_model, low_soc, battery_capacity);

        // Traverse to vertex 99 (no charging station)
        let trajectory = mock_trajectory(99);

        let charging_model = SimpleChargingModel {
            charging_station_locator: service.charging_station_locator.clone(),
            starting_soc: service.starting_soc,
            full_soc: service.full_soc,
            charge_soc_threshold: service.charge_soc_threshold,
            valid_power_types: service.valid_power_types.clone(),
        };

        charging_model
            .traverse_edge(
                (&trajectory.0, &trajectory.1, &trajectory.2),
                &mut state,
                &tree,
                &state_model,
            )
            .unwrap();

        // Check that SOC remained unchanged
        let final_soc = state_model.get_ratio(&state, fieldname::TRIP_SOC).unwrap();
        assert_eq!(final_soc, low_soc);

        // Check that no charging time was added
        let trip_time = state_model.get_time(&state, fieldname::TRIP_TIME).unwrap();
        let edge_time = state_model.get_time(&state, fieldname::EDGE_TIME).unwrap();
        assert_eq!(trip_time, Time::ZERO);
        assert_eq!(edge_time, Time::ZERO);
    }

    #[test]
    fn test_charging_with_different_power_types() {
        let service = mock_simple_charging_service();
        let state_model = state_model(service);
        let tree = SearchTree::default();

        // Test DC fast charging
        let low_soc = Ratio::new::<uom::si::ratio::percent>(15.0);
        let battery_capacity = Energy::new::<uom::si::energy::kilowatt_hour>(60.0);
        let mut state_dc = state_vector(&state_model, low_soc, battery_capacity);

        let trajectory_dc = mock_trajectory(1); // DC charging station

        let charging_model = SimpleChargingModel {
            charging_station_locator: mock_charging_station_locator(),
            starting_soc: Ratio::new::<uom::si::ratio::percent>(50.0),
            full_soc: Ratio::new::<uom::si::ratio::percent>(100.0),
            charge_soc_threshold: Ratio::new::<uom::si::ratio::percent>(20.0),
            valid_power_types: vec![PowerType::DCFC, PowerType::L2].into_iter().collect(),
        };

        charging_model
            .traverse_edge(
                (&trajectory_dc.0, &trajectory_dc.1, &trajectory_dc.2),
                &mut state_dc,
                &tree,
                &state_model,
            )
            .unwrap();
        let dc_charge_time = state_model
            .get_time(&state_dc, fieldname::TRIP_TIME)
            .unwrap();

        // Test AC Level 2 charging
        let mut state_ac = state_vector(&state_model, low_soc, battery_capacity);
        let trajectory_ac = mock_trajectory(2); // AC charging station

        charging_model
            .traverse_edge(
                (&trajectory_ac.0, &trajectory_ac.1, &trajectory_ac.2),
                &mut state_ac,
                &tree,
                &state_model,
            )
            .unwrap();
        let ac_charge_time = state_model
            .get_time(&state_ac, fieldname::TRIP_TIME)
            .unwrap();

        // DC should be faster than AC
        assert!(dc_charge_time < ac_charge_time);

        // Both should result in 100% SOC
        assert_eq!(
            state_model
                .get_ratio(&state_dc, fieldname::TRIP_SOC)
                .unwrap(),
            Ratio::new::<uom::si::ratio::percent>(100.0)
        );
        assert_eq!(
            state_model
                .get_ratio(&state_ac, fieldname::TRIP_SOC)
                .unwrap(),
            Ratio::new::<uom::si::ratio::percent>(100.0)
        );
    }

    #[test]
    fn test_invalid_power_type() {
        let service = mock_simple_charging_service();
        let state_model = state_model(service);
        let tree = SearchTree::default();

        // Set SOC to 15% (below 20% threshold)
        let low_soc = Ratio::new::<uom::si::ratio::percent>(15.0);
        let battery_capacity = Energy::new::<uom::si::energy::kilowatt_hour>(60.0);
        let mut state = state_vector(&state_model, low_soc, battery_capacity);

        // Create a model that only accepts DC charging
        let charging_model = SimpleChargingModel {
            charging_station_locator: mock_charging_station_locator(),
            starting_soc: Ratio::new::<uom::si::ratio::percent>(50.0),
            full_soc: Ratio::new::<uom::si::ratio::percent>(100.0),
            charge_soc_threshold: Ratio::new::<uom::si::ratio::percent>(20.0),
            valid_power_types: vec![PowerType::DCFC].into_iter().collect(), // Only DC
        };

        // Try to charge at L2 station (vertex 2)
        let trajectory = mock_trajectory(2);
        charging_model
            .traverse_edge(
                (&trajectory.0, &trajectory.1, &trajectory.2),
                &mut state,
                &tree,
                &state_model,
            )
            .unwrap();

        // Should not charge because L2 is not in valid_power_types
        let final_soc = state_model.get_ratio(&state, fieldname::TRIP_SOC).unwrap();
        assert_eq!(final_soc, low_soc);

        let trip_time = state_model.get_time(&state, fieldname::TRIP_TIME).unwrap();
        assert_eq!(trip_time, Time::ZERO);
    }

    #[test]
    fn test_model_name_and_features() {
        use routee_compass_core::testing::mock::traversal_model::MockUpstreamModel;

        let service = mock_simple_charging_service();

        // Create state model for building
        let input_features = service.input_features();
        let output_features = service.output_features();

        // Filter out input features that are also output features (like trip_soc)
        // Those shouldn't be mocked since the service updates them
        let output_feature_names: std::collections::HashSet<String> = output_features
            .iter()
            .map(|(name, _)| name.clone())
            .collect();
        let inputs_to_mock: Vec<InputFeature> = input_features
            .iter()
            .filter(|f| !output_feature_names.contains(&f.name()))
            .cloned()
            .collect();

        // First register mock outputs for the inputs that aren't also outputs
        let mock_output_features: Vec<(String, StateVariableConfig)> = inputs_to_mock
            .iter()
            .map(|input_feature| MockUpstreamModel::input_feature_to_output_config(input_feature))
            .collect();
        let state_model_with_mocks = Arc::new(
            StateModel::empty()
                .register(vec![], mock_output_features)
                .expect("failed to register mock features"),
        );
        let state_model = Arc::new(
            state_model_with_mocks
                .register(input_features.clone(), output_features.clone())
                .expect("failed to register features"),
        );

        let model = service
            .build(&serde_json::Value::Null, state_model)
            .expect("build failed");

        // Test model name
        assert_eq!(model.name(), "Simple Charging Model");

        // Test input features (from service)
        let input_features = service.input_features();
        assert_eq!(input_features.len(), 2);
        assert!(input_features.iter().any(|f| match f {
            InputFeature::Ratio { name, .. } => name == fieldname::TRIP_SOC,
            _ => false,
        }));
        assert!(input_features.iter().any(|f| match f {
            InputFeature::Energy { name, .. } => name == fieldname::BATTERY_CAPACITY,
            _ => false,
        }));

        // Test output features (from service)
        let output_features = service.output_features();
        assert_eq!(output_features.len(), 3);
        assert!(output_features
            .iter()
            .any(|(name, _)| name == fieldname::EDGE_TIME));
        assert!(output_features
            .iter()
            .any(|(name, _)| name == fieldname::TRIP_TIME));
        assert!(output_features
            .iter()
            .any(|(name, _)| name == fieldname::TRIP_SOC));
    }
}
