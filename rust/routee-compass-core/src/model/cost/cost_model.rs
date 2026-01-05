use super::{
    cost_ops, network::NetworkCostRate, CostAggregation, CostFeature, TraversalCost,
    VehicleCostRate,
};
use crate::algorithm::search::SearchTree;
use crate::model::cost::CostModelError;
use crate::model::network::Edge;
use crate::model::network::Vertex;
use crate::model::state::StateModel;
use crate::model::state::StateVariable;
use indexmap::IndexMap;
use itertools::Itertools;
use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;

/// implementation of a model for calculating Cost from a state transition.
/// vectorized, where each index in these vectors matches the corresponding index
/// in the state model.
pub struct CostModel {
    features: IndexMap<String, CostFeature>,
    weights_mapping: Arc<HashMap<String, f64>>,
    vehicle_rate_mapping: Arc<HashMap<String, VehicleCostRate>>,
    network_rate_mapping: Arc<HashMap<String, NetworkCostRate>>,
    cost_aggregation: CostAggregation,
}

impl CostModel {
    /// builds a cost model for a specific query.
    ///
    /// this search instance has a state model that dictates the location of each feature.
    /// here we aim to vectorize a mapping from those features into the cost weights,
    /// vehicle cost rates and network cost rates related to that feature.
    /// at runtime, we can iterate through these vectors to compute the cost.
    ///
    /// # Arguments
    /// * `displayed_costs`      - on serialization, the cost values to calculate. state variable names can be called out explicitly here that do not contribute to the total cost.
    /// * `weights`              - user-provided weighting factors for each feature
    /// * `vehicle_rate_mapping` - for each feature name, a vehicle cost rate for that feature
    /// * `network_rate_mapping` - for each feature name, a network cost rate for that feature
    /// * `cost_aggregation`     - function for aggregating each feature cost (for example, Sum)
    /// * `state_model`          - state model instance for this search
    pub fn new(
        weights_mapping: Arc<HashMap<String, f64>>,
        vehicle_rate_mapping: Arc<HashMap<String, VehicleCostRate>>,
        network_rate_mapping: Arc<HashMap<String, NetworkCostRate>>,
        cost_aggregation: CostAggregation,
        state_model: Arc<StateModel>,
    ) -> Result<CostModel, CostModelError> {
        let ignored_weights = weights_mapping
            .keys()
            .filter(|k| !state_model.contains_key(k))
            .collect_vec();
        if !ignored_weights.is_empty() {
            return Err(CostModelError::InvalidWeightNames(
                ignored_weights.iter().map(|k| k.to_string()).collect(),
                state_model.keys().cloned().collect_vec(),
            ));
        }

        let mut features = IndexMap::new();
        let mut total_weight = 0.0;

        for (name, _) in state_model.iter() {
            // always instantiate a value for each vector, diverting to default (zero-valued) if not provided
            // which has the following effect:
            // - weight: deactivates costs for this feature (product)
            // - v_rate: ignores vehicle costs for this feature (sum)
            // - n_rate: ignores network costs for this feature (sum)
            let w_opt = weights_mapping.get(name);
            let v_opt = vehicle_rate_mapping.get(name);
            let n_opt = network_rate_mapping.get(name);
            let feature = CostFeature::new(name.clone(), w_opt, v_opt, n_opt);

            total_weight += feature.weight;
            features.insert(name.clone(), feature);
        }

        if total_weight == 0.0 {
            // TODO: update this Error variant after refactor
            return Err(CostModelError::InvalidCostVariables(vec![]));
        }

        Ok(CostModel {
            features,
            weights_mapping,
            vehicle_rate_mapping,
            network_rate_mapping,
            cost_aggregation,
        })
    }

    /// calculates the total trip cost of traversing the provided edge.
    ///
    /// For accumulator features, the cost is computed as the difference between
    /// the current and previous state (delta). For non-accumulator features,
    /// the cost is computed directly from the current state value (which already
    /// represents just the edge traversal).
    pub fn traversal_cost(
        &self,
        trajectory: (&Vertex, &Edge, &Vertex),
        previous_state: &[StateVariable],
        current_state: &[StateVariable],
        tree: &SearchTree,
        state_model: &StateModel,
    ) -> Result<TraversalCost, CostModelError> {
        let mut result = TraversalCost::default();
        for (name, feature) in self.features.iter() {
            let is_accumulator = state_model.is_accumlator(name)?;

            let v_cost = if is_accumulator {
                let current_cost =
                    feature
                        .vehicle_cost_rate
                        .compute_cost(name, current_state, state_model)?;
                let previous_cost =
                    feature
                        .vehicle_cost_rate
                        .compute_cost(name, previous_state, state_model)?;
                current_cost - previous_cost
            } else {
                feature
                    .vehicle_cost_rate
                    .compute_cost(name, current_state, state_model)?
            };

            let n_cost = if is_accumulator {
                let current_network_cost = feature.network_cost_rate.network_cost(
                    trajectory,
                    current_state,
                    tree,
                    state_model,
                )?;
                let previous_network_cost = feature.network_cost_rate.network_cost(
                    trajectory,
                    previous_state,
                    tree,
                    state_model,
                )?;
                current_network_cost - previous_network_cost
            } else {
                feature.network_cost_rate.network_cost(
                    trajectory,
                    current_state,
                    tree,
                    state_model,
                )?
            };

            let cost = v_cost + n_cost;
            result.insert(cost, feature.weight);
        }
        Ok(result)
    }

    /// calculates the total trip cost of traversing the provided edge.
    pub fn estimate_cost(
        &self,
        state: &[StateVariable],
        state_model: &StateModel,
    ) -> Result<TraversalCost, CostModelError> {
        let mut result = TraversalCost::default();
        for (name, feature) in self.features.iter() {
            let v_cost = feature
                .vehicle_cost_rate
                .compute_cost(name, state, state_model)?;
            result.insert(v_cost, feature.weight);
        }
        Ok(result)
    }

    /// Serializes other information about a cost model as a JSON value.
    ///
    /// # Arguments
    ///
    /// * `state` - the state to serialize information from
    ///
    /// # Returns
    ///
    /// JSON containing information such as the units (kph, hours, etc) or other
    /// traversal info (charge events, days traveled, etc)
    pub fn serialize_cost_info(&self) -> Result<serde_json::Value, CostModelError> {
        let mut result = serde_json::Map::with_capacity(self.features.len());
        for (index, (name, feature)) in self.features.iter().enumerate() {
            let desc = cost_ops::describe_cost_feature_configuration(
                name,
                self.weights_mapping.clone(),
                self.vehicle_rate_mapping.clone(),
                self.network_rate_mapping.clone(),
            );
            result.insert(
                name.clone(),
                json![{
                    Self::WEIGHT: json![feature.weight],
                    Self::VEHICLE_RATE: json![feature.vehicle_cost_rate],
                    Self::NETWORK_RATE: json![feature.network_cost_rate.rate_type()],
                    Self::INDEX: json![index],
                    Self::DESCRIPTION: json![desc],
                }],
            );
        }

        result.insert(
            Self::COST_AGGREGATION.to_string(),
            json![self.cost_aggregation],
        );

        Ok(json![result])
    }

    const INDEX: &'static str = "index";
    const VEHICLE_RATE: &'static str = "vehicle_rate";
    const NETWORK_RATE: &'static str = "network_rate";
    const WEIGHT: &'static str = "weight";
    const COST_AGGREGATION: &'static str = "cost_aggregation";
    const DESCRIPTION: &'static str = "description";
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::algorithm::search::{Direction, SearchTree};
    use crate::model::cost::VehicleCostRate;
    use crate::model::network::{Edge, Vertex};
    use crate::model::state::{StateModel, StateVariable, StateVariableConfig};
    use crate::model::unit::{AsF64, DistanceUnit, TimeUnit};
    use std::collections::HashMap;
    use std::sync::Arc;
    use uom::si::f64::*;
    use uom::si::length::meter;
    use uom::si::time::second;

    /// Helper function to create a minimal search tree for testing
    fn create_test_tree() -> SearchTree {
        SearchTree::new(Direction::Forward)
    }

    /// Helper function to create a test trajectory
    fn create_test_trajectory() -> (Vertex, Edge, Vertex) {
        let src = Vertex::new(0, 0.0, 0.0);
        let edge = Edge::new(0, 0, 0, 1, Length::new::<meter>(100.0));
        let dst = Vertex::new(1, 1.0, 1.0);
        (src, edge, dst)
    }

    #[test]
    fn test_accumulator_feature_uses_differential() {
        // Create a state model with an accumulator feature (distance)
        let state_model = Arc::new(StateModel::new(vec![(
            "distance".to_string(),
            StateVariableConfig::Distance {
                initial: Length::new::<meter>(0.0),
                accumulator: true, // This is an accumulator
                output_unit: Some(DistanceUnit::Meters),
            },
        )]));

        // Create cost model with distance weighted
        let mut weights = HashMap::new();
        weights.insert("distance".to_string(), 1.0);

        let mut vehicle_rates = HashMap::new();
        vehicle_rates.insert(
            "distance".to_string(),
            VehicleCostRate::Raw, // Use raw to avoid unit conversion complexity
        );

        let network_rates = HashMap::new();

        let cost_model = CostModel::new(
            Arc::new(weights),
            Arc::new(vehicle_rates),
            Arc::new(network_rates),
            CostAggregation::Sum,
            state_model.clone(),
        )
        .expect("should create cost model");

        // Create states: previous = 100m, current = 150m
        // For accumulator, cost should be the difference: 150 - 100 = 50
        let previous_state = vec![StateVariable(100.0)];
        let current_state = vec![StateVariable(150.0)];

        let tree = create_test_tree();
        let (src, edge, dst) = create_test_trajectory();

        let traversal_cost = cost_model
            .traversal_cost(
                (&src, &edge, &dst),
                &previous_state,
                &current_state,
                &tree,
                &state_model,
            )
            .expect("should compute traversal cost");

        // The cost should be the differential (50m), not the absolute value (150m)
        let cost = traversal_cost.cost_component.get("distance" as &str).unwrap();
        assert_eq!(
            cost.as_f64(),
            50.0,
            "Accumulator feature should use differential cost"
        );
    }

    #[test]
    fn test_non_accumulator_feature_uses_current_state() {
        // Create a state model with a non-accumulator feature (time)
        let state_model = Arc::new(StateModel::new(vec![(
            "time".to_string(),
            StateVariableConfig::Time {
                initial: Time::new::<second>(0.0),
                accumulator: false, // This is NOT an accumulator
                output_unit: Some(TimeUnit::Seconds),
            },
        )]));

        // Create cost model with time weighted
        let mut weights = HashMap::new();
        weights.insert("time".to_string(), 1.0);

        let mut vehicle_rates = HashMap::new();
        vehicle_rates.insert(
            "time".to_string(),
            VehicleCostRate::Raw, // Use raw to avoid unit conversion complexity
        );

        let network_rates = HashMap::new();

        let cost_model = CostModel::new(
            Arc::new(weights),
            Arc::new(vehicle_rates),
            Arc::new(network_rates),
            CostAggregation::Sum,
            state_model.clone(),
        )
        .expect("should create cost model");

        // Create states: previous = 5s, current = 10s
        // For non-accumulator, cost should be the current value: 10s
        let previous_state = vec![StateVariable(5.0)];
        let current_state = vec![StateVariable(10.0)];

        let tree = create_test_tree();
        let (src, edge, dst) = create_test_trajectory();

        let traversal_cost = cost_model
            .traversal_cost(
                (&src, &edge, &dst),
                &previous_state,
                &current_state,
                &tree,
                &state_model,
            )
            .expect("should compute traversal cost");

        // The cost should be the current state value (10s), not the differential (5s)
        let cost = traversal_cost.cost_component.get("time" as &str).unwrap();
        assert_eq!(
            cost.as_f64(),
            10.0,
            "Non-accumulator feature should use current state value"
        );
    }

    #[test]
    fn test_mixed_accumulator_and_non_accumulator() {
        // Create a state model with both accumulator and non-accumulator features
        let state_model = Arc::new(StateModel::new(vec![
            (
                "distance".to_string(),
                StateVariableConfig::Distance {
                    initial: Length::new::<meter>(0.0),
                    accumulator: true, // Accumulator
                    output_unit: Some(DistanceUnit::Meters),
                },
            ),
            (
                "time".to_string(),
                StateVariableConfig::Time {
                    initial: Time::new::<second>(0.0),
                    accumulator: false, // Non-accumulator
                    output_unit: Some(TimeUnit::Seconds),
                },
            ),
        ]));

        // Create cost model with both features weighted
        let mut weights = HashMap::new();
        weights.insert("distance".to_string(), 1.0);
        weights.insert("time".to_string(), 1.0);

        let mut vehicle_rates = HashMap::new();
        vehicle_rates.insert(
            "distance".to_string(),
            VehicleCostRate::Raw, // Use raw to avoid unit conversion complexity
        );
        vehicle_rates.insert(
            "time".to_string(),
            VehicleCostRate::Raw, // Use raw to avoid unit conversion complexity
        );

        let network_rates = HashMap::new();

        let cost_model = CostModel::new(
            Arc::new(weights),
            Arc::new(vehicle_rates),
            Arc::new(network_rates),
            CostAggregation::Sum,
            state_model.clone(),
        )
        .expect("should create cost model");

        // Determine the correct state vector order based on the StateModel
        // The StateModel uses IndexMap which preserves insertion order
        let feature_order: Vec<String> = state_model.keys().cloned().collect();

        // Create states:
        // We need to match the order in the StateModel
        // distance: previous = 100m, current = 200m (diff = 100m)
        // time: previous = 5s, current = 8s (current value = 8s)
        let mut previous_state = vec![StateVariable(0.0); 2];
        let mut current_state = vec![StateVariable(0.0); 2];

        for (idx, name) in feature_order.iter().enumerate() {
            match name.as_str() {
                "distance" => {
                    previous_state[idx] = StateVariable(100.0);
                    current_state[idx] = StateVariable(200.0);
                }
                "time" => {
                    previous_state[idx] = StateVariable(5.0);
                    current_state[idx] = StateVariable(8.0);
                }
                _ => {}
            }
        }

        let tree = create_test_tree();
        let (src, edge, dst) = create_test_trajectory();

        let traversal_cost = cost_model
            .traversal_cost(
                (&src, &edge, &dst),
                &previous_state,
                &current_state,
                &tree,
                &state_model,
            )
            .expect("should compute traversal cost");

        // Check accumulator feature (distance) uses differential
        let distance_cost = traversal_cost.cost_component.get("distance" as &str).unwrap();
        assert_eq!(
            distance_cost.as_f64(),
            100.0,
            "Distance (accumulator) should use differential"
        );

        // Check non-accumulator feature (time) uses current state
        let time_cost = traversal_cost.cost_component.get("time" as &str).unwrap();
        assert_eq!(
            time_cost.as_f64(),
            8.0,
            "Time (non-accumulator) should use current state"
        );

        // Check total cost (should be sum of both)
        assert_eq!(
            traversal_cost.total_cost.as_f64(),
            108.0,
            "Total cost should be sum of both features"
        );
    }

    #[test]
    fn test_accumulator_with_zero_differential() {
        // Test case where accumulator has no change between states
        let state_model = Arc::new(StateModel::new(vec![(
            "distance".to_string(),
            StateVariableConfig::Distance {
                initial: Length::new::<meter>(0.0),
                accumulator: true,
                output_unit: Some(DistanceUnit::Meters),
            },
        )]));

        let mut weights = HashMap::new();
        weights.insert("distance".to_string(), 1.0);

        let mut vehicle_rates = HashMap::new();
        vehicle_rates.insert(
            "distance".to_string(),
            VehicleCostRate::Raw, // Use raw to avoid unit conversion complexity
        );

        let network_rates = HashMap::new();

        let cost_model = CostModel::new(
            Arc::new(weights),
            Arc::new(vehicle_rates),
            Arc::new(network_rates),
            CostAggregation::Sum,
            state_model.clone(),
        )
        .expect("should create cost model");

        // Same value in both states
        let previous_state = vec![StateVariable(100.0)];
        let current_state = vec![StateVariable(100.0)];

        let tree = create_test_tree();
        let (src, edge, dst) = create_test_trajectory();

        let traversal_cost = cost_model
            .traversal_cost(
                (&src, &edge, &dst),
                &previous_state,
                &current_state,
                &tree,
                &state_model,
            )
            .expect("should compute traversal cost");

        let cost = traversal_cost.cost_component.get("distance" as &str).unwrap();
        assert!(
            cost.as_f64() <= 1e-9,
            "Zero differential should result in MIN_COST or zero cost, got {}",
            cost.as_f64()
        );
    }

    #[test]
    fn test_accumulator_with_negative_differential() {
        // Test case where accumulator decreases (e.g., battery charge)
        let state_model = Arc::new(StateModel::new(vec![(
            "energy".to_string(),
            StateVariableConfig::Energy {
                initial: Energy::new::<uom::si::energy::joule>(0.0),
                accumulator: true,
                output_unit: None,
            },
        )]));

        let mut weights = HashMap::new();
        weights.insert("energy".to_string(), 1.0);

        let mut vehicle_rates = HashMap::new();
        vehicle_rates.insert(
            "energy".to_string(),
            VehicleCostRate::Raw, // Use raw to avoid unit conversion complexity
        );

        let network_rates = HashMap::new();

        let cost_model = CostModel::new(
            Arc::new(weights),
            Arc::new(vehicle_rates),
            Arc::new(network_rates),
            CostAggregation::Sum,
            state_model.clone(),
        )
        .expect("should create cost model");

        // Energy decreases from 1000 to 800 (consumed 200 units)
        // Differential = 800 - 1000 = -200
        let previous_state = vec![StateVariable(1000.0)];
        let current_state = vec![StateVariable(800.0)];

        let tree = create_test_tree();
        let (src, edge, dst) = create_test_trajectory();

        let traversal_cost = cost_model
            .traversal_cost(
                (&src, &edge, &dst),
                &previous_state,
                &current_state,
                &tree,
                &state_model,
            )
            .expect("should compute traversal cost");

        // Note: Cost::enforce_strictly_positive in TraversalCost::insert will make this MIN_COST (1e-10)
        // since negative costs are converted to MIN_COST
        let cost = traversal_cost.cost_component.get("energy" as &str).unwrap();
        assert!(
            cost.as_f64() <= 1e-9,
            "Negative differential is converted to MIN_COST by Cost::enforce_strictly_positive, got {}",
            cost.as_f64()
        );
    }
}
