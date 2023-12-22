use super::search_error::SearchError;
use crate::model::cost::cost_model::CostModel;
use crate::model::road_network::edge_id::EdgeId;
use crate::model::road_network::graph::Graph;
use crate::model::traversal::state::traversal_state::TraversalState;
use crate::model::traversal::traversal_model::TraversalModel;
use crate::model::unit::Cost;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::{fmt::Display, sync::RwLockReadGuard};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EdgeTraversal {
    pub edge_id: EdgeId,
    pub access_cost: Cost,
    pub traversal_cost: Cost,
    pub result_state: TraversalState,
}

impl EdgeTraversal {
    pub fn edge_cost(&self) -> Cost {
        self.access_cost + self.traversal_cost
    }
}

impl Display for EdgeTraversal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "edge {} acost:{} tcost:{} state:{:?}",
            self.edge_id, self.access_cost, self.traversal_cost, self.result_state
        )
    }
}

impl EdgeTraversal {
    /// traverses an edge, possibly after traversing some previous edge,
    /// collecting the access and traversal costs. returns the
    /// accumulated cost and updated search state.
    pub fn perform_traversal(
        edge_id: EdgeId,
        prev_edge_id: Option<EdgeId>,
        prev_state: &TraversalState,
        g: &RwLockReadGuard<Graph>,
        tm: &Arc<dyn TraversalModel>,
        um: &CostModel,
    ) -> Result<EdgeTraversal, SearchError> {
        let (src, edge, dst) = g
            .edge_triplet_attrs(edge_id)
            .map_err(SearchError::GraphError)?;

        let (access_state, access_cost) = prev_edge_id
            .map(|prev_e| {
                let prev_edge = g.get_edge(prev_e).map_err(SearchError::GraphError)?;
                let prev_src_v = g
                    .get_vertex(prev_edge.src_vertex_id)
                    .map_err(SearchError::GraphError)?;

                // we are coming from some previous edge and need to access the next edge first to proceed
                // with traversal. if there is an access state, compute the access cost.
                tm.access_edge(prev_src_v, prev_edge, src, edge, dst, prev_state)
                    .map_err(SearchError::TraversalModelFailure)
                    .and_then(|next_state_opt| match next_state_opt {
                        Some(next_state) => {
                            let cost = um
                                .access_cost(prev_edge, edge, prev_state, &next_state)
                                .map_err(SearchError::CostError)?;
                            Ok((next_state, cost))
                        }
                        None => Ok((prev_state.to_owned(), Cost::ZERO)),
                    })
            })
            .unwrap_or(Ok((prev_state.to_owned(), Cost::ZERO)))?;

        let result_state = tm
            .traverse_edge(src, edge, dst, &access_state)
            .map_err(SearchError::TraversalModelFailure)?;

        let traversal_cost = um
            .traversal_cost(edge, prev_state, &result_state)
            .map_err(SearchError::CostError)?;

        let result = EdgeTraversal {
            edge_id,
            access_cost,
            traversal_cost,
            result_state,
        };

        Ok(result)
    }
}
