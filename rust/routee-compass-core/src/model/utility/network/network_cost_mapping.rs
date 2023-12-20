use crate::model::utility::utility_error::UtilityError;
use crate::model::{
    road_network::edge_id::EdgeId,
    traversal::state::state_variable::StateVar,
    utility::{cost::Cost, cost_aggregation::CostAggregation},
};
use std::collections::HashMap;

/// a mapping for how to transform network state values into a Cost.
/// mappings come via lookup functions.
///
/// when multiple mappings are specified they are applied sequentially (in user-defined order)
/// to the state value.
pub enum NetworkCostMapping {
    EdgeLookup {
        lookup: HashMap<EdgeId, Cost>,
    },
    EdgeEdgeLookup {
        lookup: HashMap<(EdgeId, EdgeId), Cost>,
    },
    Combined(Vec<NetworkCostMapping>, CostAggregation),
}

impl NetworkCostMapping {
    pub fn traversal_cost(
        &self,
        _state: &[StateVar],
        edge_id: &EdgeId,
    ) -> Result<Cost, UtilityError> {
        match self {
            NetworkCostMapping::EdgeEdgeLookup { lookup: _ } => Ok(Cost::ZERO),
            NetworkCostMapping::EdgeLookup { lookup } => {
                let cost = lookup.get(edge_id).unwrap_or(&Cost::ZERO).to_owned();
                Ok(cost)
            }
            NetworkCostMapping::Combined(mappings, op) => {
                let mapped = mappings
                    .iter()
                    .map(|f| f.traversal_cost(_state, edge_id))
                    .collect::<Result<Vec<Cost>, UtilityError>>()?;
                let cost = op.agg(&mapped);

                Ok(cost)
            }
        }
    }

    /// maps a state variable to a Cost value based on a user-configured mapping.
    ///
    /// # Arguments
    ///
    /// * `state` - the state variable to map to a Cost value
    ///
    /// # Result
    ///
    /// the Cost value for that state, a real number that is aggregated with
    /// other Cost values in a common unit space.
    pub fn access_cost(
        &self,
        _state: &[StateVar],
        src_edge: &EdgeId,
        dst_edge: &EdgeId,
    ) -> Result<Cost, UtilityError> {
        match self {
            NetworkCostMapping::EdgeLookup { lookup: _ } => Ok(Cost::ZERO),
            NetworkCostMapping::EdgeEdgeLookup { lookup } => {
                let result = lookup.get(&(*src_edge, *dst_edge)).unwrap_or(&Cost::ZERO);
                Ok(*result)
            }
            NetworkCostMapping::Combined(mappings, op) => {
                let mapped = mappings
                    .iter()
                    .map(|f| f.access_cost(_state, src_edge, dst_edge))
                    .collect::<Result<Vec<Cost>, UtilityError>>()?;
                let cost = op.agg(&mapped);

                Ok(cost)
            }
        }
    }
}
