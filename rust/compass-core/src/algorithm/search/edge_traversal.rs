use serde::Serialize;

use crate::model::traversal::access_result::AccessResult;
use crate::model::traversal::traversal_model::TraversalModel;

use super::search_error::SearchError;
use crate::model::traversal::state::traversal_state::TraversalState;
use crate::model::{
    cost::cost::Cost,
    graph::{directed_graph::DirectedGraph, edge_id::EdgeId},
};
use std::{fmt::Display, sync::RwLockReadGuard};

#[derive(Clone, Debug, Serialize)]
pub struct EdgeTraversal {
    pub edge_id: EdgeId,
    pub access_cost: Cost,
    pub traversal_cost: Cost,
    pub result_state: TraversalState,
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
        prev_state: &TraversalState,
        g: &RwLockReadGuard<Box<dyn DirectedGraph>>,
        m: &RwLockReadGuard<Box<dyn TraversalModel>>,
    ) -> Result<EdgeTraversal, SearchError> {
        let (src, edge, dst) = g
            .edge_triplet_attrs(edge_id)
            .map_err(SearchError::GraphError)?;

        // let (access_cost, access_state);
        let access_result = match prev_edge_id {
            Some(prev_e) => {
                let prev_edge = g.edge_attr(prev_e).map_err(SearchError::GraphError)?;
                let prev_src_v = g.vertex_attr(prev_edge.src_vertex_id)?;
                m.access_cost(&prev_src_v, &prev_edge, &src, &edge, &dst, &prev_state)
            }
            None => Ok(AccessResult::no_cost(prev_state)),
        }
        .map_err(SearchError::TraversalModelFailure)?;

        let traversal_result = m
            .traversal_cost(&src, &edge, &dst, &access_result.updated_state)
            .map_err(SearchError::TraversalModelFailure)?;

        let result = EdgeTraversal {
            edge_id,
            access_cost: access_result.total_cost,
            traversal_cost: traversal_result.total_cost,
            result_state: traversal_result.updated_state,
        };

        Ok(result)
    }
}
