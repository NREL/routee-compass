use super::search_error::SearchError;
use crate::model::traversal::state::search_state::SearchState;
use crate::model::{
    cost::cost::Cost,
    graph::{directed_graph::DirectedGraph, edge_id::EdgeId},
    traversal::traversal_model::TraversalModel,
};
use std::{fmt::Display, sync::RwLockReadGuard};

#[derive(Clone)]
pub struct EdgeTraversal {
    pub edge_id: EdgeId,
    pub access_cost: Cost,
    pub traversal_cost: Cost,
    pub result_state: SearchState,
}

impl EdgeTraversal {
    pub fn edge_cost(&self) -> Cost {
        return self.access_cost + self.traversal_cost;
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
    ///
    /// traverses an edge, possibly after traversing some previous edge,
    /// collecting the access and traversal costs. returns the
    /// accumulated cost and updated search state.
    pub fn new(
        edge_id: EdgeId,
        prev_edge_id: Option<EdgeId>,
        prev_state: &SearchState,
        g: &RwLockReadGuard<&dyn DirectedGraph>,
        m: &RwLockReadGuard<&dyn TraversalModel<State = SearchState>>,
    ) -> Result<EdgeTraversal, SearchError> {
        let (src, edge, dst) = g
            .edge_triplet_attrs(edge_id)
            .map_err(SearchError::GraphCorrectnessFailure)?;

        let (access_cost, access_state);
        (access_cost, access_state) = match prev_edge_id {
            Some(prev_e) => {
                let prev_edge = g
                    .edge_attr(prev_e)
                    .map_err(SearchError::GraphCorrectnessFailure)?;
                let prev_src_v = g.vertex_attr(prev_edge.src_vertex_id)?;
                m.access_cost(&prev_src_v, &prev_edge, &src, &edge, &dst, &prev_state)
            }
            None => Ok((Cost::ZERO, prev_state.to_vec())),
        }
        .map_err(SearchError::TraversalModelFailure)?;

        let (traversal_cost, result_state) = m
            .traversal_cost(&src, &edge, &dst, &access_state)
            .map_err(SearchError::TraversalModelFailure)?;

        let result = EdgeTraversal {
            edge_id,
            access_cost,
            traversal_cost,
            result_state,
        };

        Ok(result)
    }
}
