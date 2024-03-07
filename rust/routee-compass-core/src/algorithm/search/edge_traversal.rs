use super::search_error::SearchError;
use super::search_instance::SearchInstance;
use crate::model::road_network::edge_id::EdgeId;
use crate::model::traversal::state::state_variable::StateVar;
use crate::model::traversal::state::traversal_state::TraversalState;
use crate::model::unit::Cost;
use allocative::Allocative;
use serde::{Deserialize, Serialize};
use std::fmt::Display;

#[derive(Clone, Debug, Serialize, Deserialize, Allocative)]
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
        prev_state: &[StateVar],
        si: &SearchInstance,
    ) -> Result<EdgeTraversal, SearchError> {
        let mut result_state = prev_state.to_vec();
        let mut access_cost = Cost::ZERO;

        // find this traversal in the graph
        let (src, edge, dst) = si
            .directed_graph
            .edge_triplet_attrs(edge_id)
            .map_err(SearchError::GraphError)?;

        // perform access traversal + access cost if a previous edge exists
        if let Some(prev_e) = prev_edge_id {
            let prev_edge = si
                .directed_graph
                .get_edge(prev_e)
                .map_err(SearchError::GraphError)?;
            let prev_src_v = si
                .directed_graph
                .get_vertex(prev_edge.src_vertex_id)
                .map_err(SearchError::GraphError)?;

            si.traversal_model.access_edge(
                prev_src_v,
                prev_edge,
                src,
                edge,
                dst,
                &mut result_state,
            )?;

            let ac = si
                .cost_model
                .access_cost(prev_edge, edge, prev_state, &result_state)
                .map_err(SearchError::CostError)?;
            access_cost = access_cost + ac;
        }

        si.traversal_model
            .traverse_edge(src, edge, dst, &mut result_state)
            .map_err(SearchError::TraversalModelFailure)?;

        let traversal_cost = si
            .cost_model
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
