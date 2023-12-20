use super::cost_aggregation::CostAggregation;
use super::vehicle::vehicle_cost_ops;
use super::vehicle::vehicle_utility_mapping::VehicleUtilityMapping;
use crate::model::property::edge::Edge;
use crate::model::traversal::state::state_variable::StateVar;
use crate::model::utility::cost::Cost;
use crate::model::utility::utility_error::UtilityError;
use std::collections::HashMap;
use std::sync::Arc;

pub struct UtilityModel {
    dimensions: Vec<(String, usize)>,
    vehicle_mapping: Arc<HashMap<String, VehicleUtilityMapping>>,
    cost_aggregation: CostAggregation,
}

impl UtilityModel {
    pub fn new(
        dimensions: Vec<(String, usize)>,
        vehicle_mapping: Arc<HashMap<String, VehicleUtilityMapping>>,
        cost_aggregation: CostAggregation,
    ) -> UtilityModel {
        UtilityModel {
            dimensions,
            vehicle_mapping,
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
    fn traversal_cost(
        &self,
        _edge: &Edge,
        prev_state: &[StateVar],
        next_state: &[StateVar],
    ) -> Result<Cost, UtilityError> {
        let vehicle_costs = vehicle_cost_ops::calculate_vehicle_cost(
            prev_state,
            next_state,
            &self.dimensions,
            self.vehicle_mapping.clone(),
        )?;
        let vehicle_cost = self.cost_aggregation.agg(&vehicle_costs);
        Ok(vehicle_cost)
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
    fn access_cost(
        &self,
        _prev_edge: Option<&Edge>,
        _next_edge: &Edge,
        prev_state: &[StateVar],
        next_state: &[StateVar],
    ) -> Result<Cost, UtilityError> {
        let vehicle_costs = vehicle_cost_ops::calculate_vehicle_cost(
            prev_state,
            next_state,
            &self.dimensions,
            self.vehicle_mapping.clone(),
        )?;
        let vehicle_cost = self.cost_aggregation.agg(&vehicle_costs);
        Ok(vehicle_cost)
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
    fn cost_estimate(
        &self,
        src_state: &[StateVar],
        dst_state: &[StateVar],
    ) -> Result<Cost, UtilityError> {
        let vehicle_costs = vehicle_cost_ops::calculate_vehicle_cost(
            src_state,
            dst_state,
            &self.dimensions,
            self.vehicle_mapping.clone(),
        )?;
        let vehicle_cost = self.cost_aggregation.agg(&vehicle_costs);
        Ok(vehicle_cost)
    }
}
