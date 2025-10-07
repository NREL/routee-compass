use crate::model::network::{Edge, Vertex, VertexId};
use crate::model::state::{StateModel, StateVariable};
use crate::model::unit::Cost;
use crate::model::{cost::CostModelError, network::EdgeId};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

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
    VertexLookup {
        lookup: HashMap<VertexId, Cost>,
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
            NetworkCostRate::VertexLookup { lookup: _ } => Ok(Cost::ZERO),
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

    /// computes the cost for accessing this part of the network.
    pub fn network_cost(
        &self,
        trajectory: (&Vertex, &Edge, &Vertex),
        state: &[StateVariable],
        state_model: Arc<StateModel>
    ) -> Result<Cost, CostModelError> {
        match self {
            NetworkCostRate::Zero => Ok(Cost::ZERO),
            NetworkCostRate::EdgeLookup { lookup } => {
                let (_, edge, _) = trajectory;
                let cost = lookup.get(&edge.edge_id).copied().unwrap_or_default();
                Ok(cost)
            },
            NetworkCostRate::VertexLookup { lookup } => {
                let (src, _, _) = trajectory;
                let cost = lookup.get(&src.vertex_id).copied().unwrap_or_default();
                Ok(cost)
            },
            NetworkCostRate::Combined(rates) => {
                let mapped = rates
                    .iter()
                    .map(|f| f.network_cost(trajectory, state, state_model.clone()))
                    .collect::<Result<Vec<Cost>, CostModelError>>()?;
                let cost = mapped.iter().fold(Cost::ZERO, |a, b| a + *b);

                Ok(cost)
            },
        }
    }
}
