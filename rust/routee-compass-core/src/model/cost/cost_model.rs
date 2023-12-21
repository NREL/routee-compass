use super::cost_aggregation::CostAggregation;
use super::cost_ops;
use super::network::network_cost_rate::NetworkCostRate;
use super::vehicle::vehicle_cost_rate::VehicleCostRate;
use crate::model::cost::cost_error::CostError;
use crate::model::property::edge::Edge;
use crate::model::traversal::state::state_variable::StateVar;
use crate::model::unit::Cost;
use std::collections::HashMap;
use std::sync::Arc;

/// implementation of a model for calculating Cost from a state transition.
pub struct CostModel {
    state_variable_indices: Vec<(String, usize)>,
    state_variable_coefficients: Arc<HashMap<String, f64>>,
    vehicle_state_variable_rates: Arc<HashMap<String, VehicleCostRate>>,
    network_state_variable_rates: Arc<HashMap<String, NetworkCostRate>>,
    cost_aggregation: CostAggregation,
}

impl CostModel {
    pub fn new(
        state_variable_indices: Vec<(String, usize)>,
        state_variable_coefficients: Arc<HashMap<String, f64>>,
        vehicle_state_variable_rates: Arc<HashMap<String, VehicleCostRate>>,
        network_state_variable_rates: Arc<HashMap<String, NetworkCostRate>>,
        cost_aggregation: CostAggregation,
    ) -> CostModel {
        CostModel {
            state_variable_indices,
            state_variable_coefficients,
            vehicle_state_variable_rates,
            network_state_variable_rates,
            cost_aggregation,
        }
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
        let vehicle_costs = cost_ops::calculate_vehicle_costs(
            prev_state,
            next_state,
            &self.state_variable_indices,
            self.state_variable_coefficients.clone(),
            self.vehicle_state_variable_rates.clone(),
        )?;
        let vehicle_cost = self.cost_aggregation.agg(&vehicle_costs);
        let network_costs = cost_ops::calculate_network_traversal_costs(
            prev_state,
            next_state,
            edge,
            &self.state_variable_indices,
            self.state_variable_coefficients.clone(),
            self.network_state_variable_rates.clone(),
        )?;
        let network_cost = self.cost_aggregation.agg(&network_costs);
        Ok(vehicle_cost + network_cost)
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
        let vehicle_costs = cost_ops::calculate_vehicle_costs(
            prev_state,
            next_state,
            &self.state_variable_indices,
            self.state_variable_coefficients.clone(),
            self.vehicle_state_variable_rates.clone(),
        )?;
        let vehicle_cost = self.cost_aggregation.agg(&vehicle_costs);
        let network_costs = cost_ops::calculate_network_access_costs(
            prev_state,
            next_state,
            prev_edge,
            next_edge,
            &self.state_variable_indices,
            self.state_variable_coefficients.clone(),
            self.network_state_variable_rates.clone(),
        )?;
        let network_cost = self.cost_aggregation.agg(&network_costs);
        Ok(vehicle_cost + network_cost)
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
    /// Either a cost estimate or an error.
    pub fn cost_estimate(
        &self,
        src_state: &[StateVar],
        dst_state: &[StateVar],
    ) -> Result<Cost, CostError> {
        let vehicle_costs = cost_ops::calculate_vehicle_costs(
            src_state,
            dst_state,
            &self.state_variable_indices,
            self.state_variable_coefficients.clone(),
            self.vehicle_state_variable_rates.clone(),
        )?;
        let vehicle_cost = self.cost_aggregation.agg(&vehicle_costs);
        Ok(vehicle_cost)
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
}
