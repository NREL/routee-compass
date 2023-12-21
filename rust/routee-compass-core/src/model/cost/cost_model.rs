use super::cost_aggregation::CostAggregation;
use super::cost_ops;
use super::network::network_cost_mapping::NetworkUtilityMapping;
use super::vehicle::vehicle_cost_mapping::VehicleUtilityMapping;
use crate::model::cost::cost_error::CostError;
use crate::model::property::edge::Edge;
use crate::model::traversal::state::state_variable::StateVar;
use crate::model::unit::Cost;
use std::collections::HashMap;
use std::sync::Arc;

/// implementation of a model for calculating Cost from a state transition.
pub struct CostModel {
    dimensions: Vec<(String, usize)>,
    vehicle_mapping: Arc<HashMap<String, VehicleUtilityMapping>>,
    network_mapping: Arc<HashMap<String, NetworkUtilityMapping>>,
    cost_aggregation: CostAggregation,
}

impl CostModel {
    pub fn new(
        dimensions: Vec<(String, usize)>,
        vehicle_mapping: Arc<HashMap<String, VehicleUtilityMapping>>,
        network_mapping: Arc<HashMap<String, NetworkUtilityMapping>>,
        cost_aggregation: CostAggregation,
    ) -> CostModel {
        CostModel {
            dimensions,
            vehicle_mapping,
            network_mapping,
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
            &self.dimensions,
            self.vehicle_mapping.clone(),
        )?;
        let vehicle_cost = self.cost_aggregation.agg(&vehicle_costs);
        let network_costs = cost_ops::calculate_network_traversal_costs(
            prev_state,
            next_state,
            edge,
            &self.dimensions,
            self.network_mapping.clone(),
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
            &self.dimensions,
            self.vehicle_mapping.clone(),
        )?;
        let vehicle_cost = self.cost_aggregation.agg(&vehicle_costs);
        let network_costs = cost_ops::calculate_network_access_costs(
            prev_state,
            next_state,
            prev_edge,
            next_edge,
            &self.dimensions,
            self.network_mapping.clone(),
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
            &self.dimensions,
            self.vehicle_mapping.clone(),
        )?;
        let vehicle_cost = self.cost_aggregation.agg(&vehicle_costs);
        Ok(vehicle_cost)
    }
}
