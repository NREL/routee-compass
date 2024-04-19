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
    pub fn total_cost(&self) -> Cost {
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
        next_edge_id: EdgeId,
        prev_edge_id: Option<EdgeId>,
        prev_state: &[StateVar],
        si: &SearchInstance,
    ) -> Result<EdgeTraversal, SearchError> {
        let mut result_state = prev_state.to_vec();
        let mut access_cost = Cost::ZERO;

        // find this traversal in the graph
        let traversal_trajectory = si
            .directed_graph
            .edge_triplet_attrs(next_edge_id)
            .map_err(SearchError::GraphError)?;

        // perform access traversal + access cost if a previous edge exists
        // regardless of search direction, we grab the forward-oriented trajectory
        // for computing costs: (v1)-[prev]->(v2)-[next]->(v3)
        if let Some(prev_edge_id) = prev_edge_id {
            let e1 = si
                .directed_graph
                .get_edge(prev_edge_id)
                .map_err(SearchError::GraphError)?;
            let v1 = si
                .directed_graph
                .get_vertex(e1.src_vertex_id)
                .map_err(SearchError::GraphError)?;

            let (v2, e2, v3) = traversal_trajectory;
            let access_trajectory = (v1, e1, v2, e2, v3);

            si.access_model
                .access_edge(access_trajectory, &mut result_state, &si.state_model)?;

            let ac = si
                .cost_model
                .access_cost(e1, e2, prev_state, &result_state)
                .map_err(SearchError::CostError)?;
            access_cost = access_cost + ac;
        }

        si.traversal_model
            .traverse_edge(traversal_trajectory, &mut result_state, &si.state_model)
            .map_err(SearchError::TraversalModelFailure)?;

        let (_, edge, _) = traversal_trajectory;
        let total_cost = si
            .cost_model
            .traversal_cost(edge, prev_state, &result_state)
            .map_err(SearchError::CostError)?;
        let traversal_cost = total_cost - access_cost;

        let result = EdgeTraversal {
            edge_id: next_edge_id,
            access_cost,
            traversal_cost,
            result_state,
        };

        Ok(result)
    }
}
