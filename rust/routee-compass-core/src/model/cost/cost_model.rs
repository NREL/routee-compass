use super::cost_aggregation::CostAggregation;
use super::cost_ops;
use super::network::network_cost_rate::NetworkCostRate;
use super::vehicle::vehicle_cost_rate::VehicleCostRate;
use crate::model::cost::cost_error::CostError;
use crate::model::property::edge::Edge;
use crate::model::state::state_model::StateModel;
use crate::model::traversal::state::state_variable::StateVar;

use crate::model::unit::Cost;
use std::collections::HashMap;
use std::sync::Arc;

/// implementation of a model for calculating Cost from a state transition.
pub struct CostModel {
    state_variable_indices: Vec<(String, usize)>,
    state_variable_coefficients: Vec<f64>,
    vehicle_state_variable_rates: Vec<VehicleCostRate>,
    network_state_variable_rates: Vec<NetworkCostRate>,
    cost_aggregation: CostAggregation,
}

impl CostModel {
    /// builds a cost model for a specific query.
    ///
    /// at configuration time, a list of rates are specified. at query time,
    /// a specialized cost model is built that takes the state variable indices from
    /// the specialized traversal model and creates a mapping into this cost
    /// model instance.
    ///
    pub fn new(
        state_variable_indices: Vec<(String, usize)>,
        state_variable_coefficients_map: Arc<HashMap<String, f64>>,
        state_model: Arc<StateModel>,
        vehicle_state_variable_rates_map: Arc<HashMap<String, VehicleCostRate>>,
        network_state_variable_rates_map: Arc<HashMap<String, NetworkCostRate>>,
        cost_aggregation: CostAggregation,
    ) -> Result<CostModel, CostError> {
        if state_variable_indices.is_empty() {
            return Err(CostError::InvalidConfiguration(String::from(
                "no state variables listed",
            )));
        }

        let mut state_variable_coefficients = vec![];
        let mut vehicle_state_variable_rates = vec![];
        let mut network_state_variable_rates = vec![];

        // map the state variable coefficiencies and rates to the state variable indices
        for (name, _state_idx) in state_variable_indices.iter() {
            let coef = state_variable_coefficients_map.get(name).ok_or_else(|| {
                CostError::InvalidConfiguration(format!("coefficient for {} not provided", name))
            })?;
            let v_rate = vehicle_state_variable_rates_map
                .get(name)
                .cloned()
                .ok_or_else(|| {
                    CostError::InvalidConfiguration(format!(
                        "vehicle rate for {} not provided",
                        name
                    ))
                })?;
            let n_rate = network_state_variable_rates_map
                .get(name)
                .cloned()
                .unwrap_or_default();

            state_variable_coefficients.push(*coef);
            vehicle_state_variable_rates.push(v_rate.clone());
            network_state_variable_rates.push(n_rate.clone());
        }

        if state_variable_coefficients.iter().sum::<f64>() == 0.0 {
            return Err(CostError::InvalidCostVariables);
        }
        Ok(CostModel {
            state_variable_indices,
            state_variable_coefficients,
            vehicle_state_variable_rates,
            network_state_variable_rates,
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
        prev_state: &[StateVar],
        next_state: &[StateVar],
    ) -> Result<Cost, CostError> {
        let vehicle_cost = cost_ops::calculate_vehicle_costs(
            prev_state,
            next_state,
            &self.state_variable_indices,
            &self.state_variable_coefficients,
            &self.vehicle_state_variable_rates,
            &self.cost_aggregation,
        )?;
        let network_cost = cost_ops::calculate_network_traversal_costs(
            prev_state,
            next_state,
            edge,
            &self.state_variable_indices,
            &self.state_variable_coefficients,
            &self.network_state_variable_rates,
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
        prev_state: &[StateVar],
        next_state: &[StateVar],
    ) -> Result<Cost, CostError> {
        let vehicle_cost = cost_ops::calculate_vehicle_costs(
            prev_state,
            next_state,
            &self.state_variable_indices,
            &self.state_variable_coefficients,
            &self.vehicle_state_variable_rates,
            &self.cost_aggregation,
        )?;
        let network_cost = cost_ops::calculate_network_access_costs(
            prev_state,
            next_state,
            prev_edge,
            next_edge,
            &self.state_variable_indices,
            &self.state_variable_coefficients,
            &self.network_state_variable_rates,
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
        src_state: &[StateVar],
        dst_state: &[StateVar],
    ) -> Result<Cost, CostError> {
        let vehicle_cost = cost_ops::calculate_vehicle_costs(
            src_state,
            dst_state,
            &self.state_variable_indices,
            &self.state_variable_coefficients,
            &self.vehicle_state_variable_rates,
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
    fn serialize_cost(&self, state: &[StateVar]) -> Result<serde_json::Value, CostError> {
        let mut state_variable_costs = self
            .state_variable_indices
            .iter()
            .map(move |(name, idx)| {
                let state_var = state
                    .get(*idx)
                    .ok_or_else(|| CostError::StateIndexOutOfBounds(*idx, name.clone()))?;

                let rate = self.vehicle_state_variable_rates.get(*idx).ok_or_else(|| {
                    let alternatives = self
                        .state_variable_indices
                        .iter()
                        .filter(|(_, idx)| *idx < self.vehicle_state_variable_rates.len())
                        .map(|(n, _)| n.to_string())
                        .collect::<Vec<_>>()
                        .join(",");
                    CostError::StateVariableNotFound(
                        name.clone(),
                        String::from("vehicle cost rates while serializing cost"),
                        alternatives,
                    )
                })?;
                let cost = rate.map_value(*state_var);
                Ok((name.clone(), cost))
            })
            .collect::<Result<HashMap<String, Cost>, CostError>>()?;

        let total_cost = state_variable_costs
            .values()
            .fold(Cost::ZERO, |a, b| a + *b);
        state_variable_costs.insert(String::from("total_cost"), total_cost);

        let result = serde_json::json!(state_variable_costs);

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
    pub fn serialize_cost_info(&self) -> serde_json::Value {
        serde_json::json!({
            "state_variable_indices": serde_json::json!(self.state_variable_indices),
            "state_variable_coefficients": serde_json::json!(*self.state_variable_coefficients),
            "vehicle_state_variable_rates": serde_json::json!(*self.vehicle_state_variable_rates),
            "network_state_variable_rates": serde_json::json!(*self.network_state_variable_rates),
            "cost_aggregation": serde_json::json!(self.cost_aggregation)
        })
    }

    /// Serialization function called by Compass output processing code that
    /// writes both the costs and the cost info to a JSON value.
    ///
    /// # Arguments
    ///
    /// * `state` - the state to serialize information from
    ///
    /// # Returns
    ///
    /// JSON containing the cost values and info described in `serialize_cost`
    /// and `serialize_cost_info`.
    pub fn serialize_cost_with_info(
        &self,
        state: &[StateVar],
    ) -> Result<serde_json::Value, CostError> {
        let mut output = serde_json::Map::new();
        output.insert(String::from("cost"), self.serialize_cost(state)?);
        output.insert(String::from("info"), self.serialize_cost_info());
        Ok(serde_json::json!(output))
    }
}
