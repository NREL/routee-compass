use super::{cost_ops, network::NetworkCostRate, CostAggregation, VehicleCostRate};
use crate::model::cost::CostModelError;
use crate::model::network::Edge;
use crate::model::state::StateModel;
use crate::model::state::StateVariable;
use crate::model::unit::Cost;
use itertools::Itertools;
use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;

/// implementation of a model for calculating Cost from a state transition.
/// vectorized, where each index in these vectors matches the corresponding index
/// in the state model.
pub struct CostModel {
    feature_indices: Vec<(String, usize)>,
    weights: Vec<f64>,
    vehicle_rates: Vec<VehicleCostRate>,
    network_rates: Vec<NetworkCostRate>,
    cost_aggregation: CostAggregation,
}

impl CostModel {
    const VEHICLE_RATES: &'static str = "vehicle_rates";
    const NETWORK_RATES: &'static str = "network_rates";
    // const FEATURES: &'static str = "features";
    const WEIGHTS: &'static str = "weights";
    const VEHICLE_RATE: &'static str = "vehicle_rate";
    const NETWORK_RATE: &'static str = "network_rate";
    const FEATURE: &'static str = "feature";
    const WEIGHT: &'static str = "weight";
    const COST_AGGREGATION: &'static str = "cost_aggregation";

    /// builds a cost model for a specific query.
    ///
    /// this search instance has a state model that dictates the location of each feature.
    /// here we aim to vectorize a mapping from those features into the cost weights,
    /// vehicle cost rates and network cost rates related to that feature.
    /// at runtime, we can iterate through these vectors to compute the cost.
    ///
    /// # Arguments
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

        let mut indices = vec![];
        let mut weights = vec![];
        let mut vehicle_rates = vec![];
        let mut network_rates = vec![];

        for (index, (name, _)) in state_model.indexed_iter() {
            // always instantiate a value for each vector, diverting to default (zero-valued) if not provided
            // which has the following effect:
            // - weight: deactivates costs for this feature (product)
            // - v_rate: ignores vehicle costs for this feature (sum)
            // - n_rate: ignores network costs for this feature (sum)
            let weight = weights_mapping.get(name).cloned().unwrap_or_default();
            let v_rate = vehicle_rate_mapping.get(name).cloned().unwrap_or_default();
            let n_rate = network_rate_mapping.get(name).cloned().unwrap_or_default();

            indices.push((name.clone(), index));
            weights.push(weight);
            vehicle_rates.push(v_rate.clone());
            network_rates.push(n_rate.clone());
        }

        if weights.iter().sum::<f64>() == 0.0 {
            return Err(CostModelError::InvalidCostVariables(weights));
        }
        Ok(CostModel {
            feature_indices: indices,
            weights,
            vehicle_rates,
            network_rates,
            cost_aggregation,
        })
    }

    /// Calculates the cost of traversing an edge due to some state transition.
    ///
    /// # Arguments
    ///
    /// * `edge` - edge traversed
    /// * `prev_state` - state of the search at the beginning of this edge
    /// * `next_state` - state of the search at the end of this edge
    ///
    /// # Returns
    ///
    /// Either a traversal cost or an error.
    pub fn traversal_cost(
        &self,
        edge: &Edge,
        prev_state: &[StateVariable],
        next_state: &[StateVariable],
    ) -> Result<Cost, CostModelError> {
        let vehicle_cost = cost_ops::calculate_vehicle_costs(
            (prev_state, next_state),
            &self.feature_indices,
            &self.weights,
            &self.vehicle_rates,
            &self.cost_aggregation,
        )?;
        let network_cost = cost_ops::calculate_network_traversal_costs(
            (prev_state, next_state),
            edge,
            &self.feature_indices,
            &self.weights,
            &self.network_rates,
            &self.cost_aggregation,
        )?;
        let total_cost = vehicle_cost + network_cost;
        let pos_cost = Cost::enforce_strictly_positive(total_cost);
        Ok(pos_cost)
    }

    /// Calculates the cost of accessing some destination edge when coming
    /// from some previous edge.
    ///
    /// These arguments appear in the network as:
    /// `() -[prev]-> () -[next]-> ()`
    /// Where `next` is the edge we want to access.
    ///
    /// # Arguments
    ///
    /// * `prev_edge` - previous edge
    /// * `next_edge` - edge we are determining the cost to access
    /// * `prev_state` - state of the search at the beginning of this edge
    /// * `next_state` - state of the search at the end of this edge
    ///
    /// # Returns
    ///
    /// Either an access result or an error.
    pub fn access_cost(
        &self,
        prev_edge: &Edge,
        next_edge: &Edge,
        prev_state: &[StateVariable],
        next_state: &[StateVariable],
    ) -> Result<Cost, CostModelError> {
        let vehicle_cost = cost_ops::calculate_vehicle_costs(
            (prev_state, next_state),
            &self.feature_indices,
            &self.weights,
            &self.vehicle_rates,
            &self.cost_aggregation,
        )?;
        let network_cost = cost_ops::calculate_network_access_costs(
            (prev_state, next_state),
            (prev_edge, next_edge),
            &self.feature_indices,
            &self.weights,
            &self.network_rates,
            &self.cost_aggregation,
        )?;
        let total_cost = vehicle_cost + network_cost;
        let pos_cost = Cost::enforce_strictly_positive(total_cost);
        Ok(pos_cost)
    }

    /// Calculates a cost estimate for traversing between a source and destination
    /// vertex without actually doing the work of traversing the edges.
    /// This estimate is used in search algorithms such as a-star algorithm, where
    /// the estimate is used to inform search order.
    ///
    /// # Arguments
    ///
    /// * `src_state` - state at source vertex
    /// * `dst_state` - estimated state at destination vertex
    ///
    /// # Returns
    ///
    /// Either a cost estimate or an error. cost estimates may be
    pub fn cost_estimate(
        &self,
        src_state: &[StateVariable],
        dst_state: &[StateVariable],
    ) -> Result<Cost, CostModelError> {
        let vehicle_cost = cost_ops::calculate_vehicle_costs(
            (src_state, dst_state),
            &self.feature_indices,
            &self.weights,
            &self.vehicle_rates,
            &self.cost_aggregation,
        )?;
        let pos_cost = Cost::enforce_non_negative(vehicle_cost);
        Ok(pos_cost)
    }

    /// Serializes the cost of a traversal state into a JSON value.
    ///
    /// # Arguments
    ///
    /// * `state` - the state to serialize
    ///
    /// # Returns
    ///
    /// A JSON serialized version of the state. This does not need to include
    /// additional details such as the units (kph, hours, etc), which can be
    /// summarized in the serialize_state_info method.
    pub fn serialize_cost(
        &self,
        state: &[StateVariable],
        state_model: Arc<StateModel>,
    ) -> Result<serde_json::Value, CostModelError> {
        // for each feature, if it is an accumulator, compute its cost
        let mut state_variable_costs = HashMap::new();
        let iter = self
            .feature_indices
            .iter()
            .filter(|(name, _)| state_model.is_accumlator(name).unwrap_or_default());
        for (name, idx) in iter {
            let state_var = state
                .get(*idx)
                .ok_or_else(|| CostModelError::StateIndexOutOfBounds(*idx, name.clone()))?;

            let rate = self.vehicle_rates.get(*idx).ok_or_else(|| {
                let alternatives = self
                    .feature_indices
                    .iter()
                    .filter(|(_, idx)| *idx < self.vehicle_rates.len())
                    .map(|(n, _)| n.to_string())
                    .collect::<Vec<_>>()
                    .join(",");
                CostModelError::StateVariableNotFound(
                    name.clone(),
                    String::from("vehicle cost rates while serializing cost"),
                    alternatives,
                )
            })?;
            match rate.map_value(*state_var) {
                Some(cost) => {
                    state_variable_costs.insert(name.clone(), cost);
                }
                None => {
                    // if the rate is zero, we don't need to include it in the cost serialization,
                    // as this means that, while it is an accumulator, it has no vehicle cost factor.
                }
            }
        }

        let total_cost = state_variable_costs
            .values()
            .fold(Cost::ZERO, |a, b| a + *b);
        state_variable_costs.insert(String::from("total_cost"), total_cost);

        let result = json!(state_variable_costs);

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
        let mut result = serde_json::Map::with_capacity(self.feature_indices.len());
        for (name, index) in self.feature_indices.iter() {
            let weight = self
                .weights
                .get(*index)
                .ok_or(CostModelError::CostVectorOutOfBounds(
                    *index,
                    String::from(Self::WEIGHTS),
                ))?;
            let veh_rate =
                self.vehicle_rates
                    .get(*index)
                    .ok_or(CostModelError::CostVectorOutOfBounds(
                        *index,
                        String::from(Self::VEHICLE_RATES),
                    ))?;
            let net_rate =
                self.network_rates
                    .get(*index)
                    .ok_or(CostModelError::CostVectorOutOfBounds(
                        *index,
                        String::from(Self::NETWORK_RATES),
                    ))?;
            result.insert(
                name.clone(),
                json![{
                    Self::FEATURE: json![name],
                    Self::WEIGHT: json![weight],
                    Self::VEHICLE_RATE: json![veh_rate],
                    Self::NETWORK_RATE: json![net_rate],
                }],
            );
        }

        result.insert(
            Self::COST_AGGREGATION.to_string(),
            json![self.cost_aggregation],
        );

        Ok(json![result])
    }
}
