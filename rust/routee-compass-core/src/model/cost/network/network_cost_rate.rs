use crate::model::network::Edge;
use crate::model::state::StateVariable;
use crate::model::unit::Cost;
use crate::model::{cost::CostModelError, network::EdgeId};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// a mapping for how to transform network state values into a Cost.
/// mappings come via lookup functions.
///
/// when multiple mappings are specified they are applied sequentially (in user-defined order)
/// to the state value.
#[derive(Serialize, Deserialize, Clone, Default, Debug)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum NetworkCostRate {
    #[default]
    Zero,
    EdgeLookup {
        lookup: HashMap<EdgeId, Cost>,
    },
    EdgeEdgeLookup {
        lookup: HashMap<(EdgeId, EdgeId), Cost>,
    },
    Combined(Vec<NetworkCostRate>),
}

impl NetworkCostRate {
    pub fn traversal_cost(
        &self,
        _prev_state_var: StateVariable,
        _next_state_var: StateVariable,
        edge: &Edge,
    ) -> Result<Cost, CostModelError> {
        match self {
            NetworkCostRate::Zero => Ok(Cost::ZERO),
            NetworkCostRate::EdgeEdgeLookup { lookup: _ } => Ok(Cost::ZERO),
            NetworkCostRate::EdgeLookup { lookup } => {
                let cost = lookup.get(&edge.edge_id).unwrap_or(&Cost::ZERO).to_owned();
                Ok(cost)
            }
            NetworkCostRate::Combined(mappings) => {
                let mapped = mappings
                    .iter()
                    .map(|f| f.traversal_cost(_prev_state_var, _next_state_var, edge))
                    .collect::<Result<Vec<Cost>, CostModelError>>()?;
                let cost = mapped.iter().fold(Cost::ZERO, |a, b| a + *b);

                Ok(cost)
            }
        }
    }

    /// maps a state variable to a Cost value based on a user-configured mapping.
    ///
    /// # Arguments
    ///
    /// * `prev_state_var` - the state variable before accessing the next edge origin
    /// * `next_state_var` - the state variable after accessing the next edge origin
    /// * `prev_edge` - the edge traversed to reach the next_edge (or none if at origin)
    /// * `next_edge` - the edge we are attempting to access (not yet traversed)
    ///
    /// # Result
    ///
    /// the Cost value for that state, a real number that is aggregated with
    /// other Cost values in a common unit space.
    pub fn access_cost(
        &self,
        _prev_state_var: StateVariable,
        _next_state_var: StateVariable,
        prev_edge: &Edge,
        next_edge: &Edge,
    ) -> Result<Cost, CostModelError> {
        match self {
            NetworkCostRate::Zero => Ok(Cost::ZERO),
            NetworkCostRate::EdgeLookup { lookup: _ } => Ok(Cost::ZERO),
            NetworkCostRate::EdgeEdgeLookup { lookup } => {
                let result = lookup
                    .get(&(prev_edge.edge_id, next_edge.edge_id))
                    .unwrap_or(&Cost::ZERO);
                Ok(*result)
            }
            NetworkCostRate::Combined(mappings) => {
                let mapped = mappings
                    .iter()
                    .map(|f| f.access_cost(_prev_state_var, _next_state_var, prev_edge, next_edge))
                    .collect::<Result<Vec<Cost>, CostModelError>>()?;
                let cost = mapped.iter().fold(Cost::ZERO, |a, b| a + *b);

                Ok(cost)
            }
        }
    }
}
