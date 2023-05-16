use std::{fmt::Display, sync::RwLockReadGuard};

use crate::model::{
    cost::cost::Cost,
    graph::{directed_graph::DirectedGraph, edge_id::EdgeId},
    traversal::traversal_model::TraversalModel,
};

use super::search_error::SearchError;

#[derive(Clone, Copy)]
pub struct EdgeTraversal<S: Copy + Clone> {
    pub access_cost: Cost,
    pub traversal_cost: Cost,
    pub result_state: S,
}

impl<S: Display + Copy> Display for EdgeTraversal<S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "acost:{} tcost:{} state:{}",
            self.access_cost, self.traversal_cost, self.result_state
        )
    }
}

impl<S: Sync + Send + Eq + Copy + Clone> EdgeTraversal<S> {
    ///
    /// traverses an edge, possibly after traversing some previous edge,
    /// collecting the access and traversal costs. returns the
    /// accumulated cost and updated search state.
    pub fn new(
        edge_id: EdgeId,
        prev_edge_id: Option<EdgeId>,
        prev_state: S,
        g: &RwLockReadGuard<&dyn DirectedGraph>,
        m: &RwLockReadGuard<&dyn TraversalModel<State = S>>,
    ) -> Result<EdgeTraversal<S>, SearchError> {
        let edge = g
            .edge_attr(edge_id)
            .map_err(SearchError::GraphCorrectnessFailure)?;

        let (access_cost, access_state);
        (access_cost, access_state) = match prev_edge_id {
            Some(prev_e) => {
                let prev_edge = g
                    .edge_attr(prev_e)
                    .map_err(SearchError::GraphCorrectnessFailure)?;
                m.access_cost(&prev_edge, &edge, &prev_state)
            }
            None => Ok((Cost::ZERO, prev_state)),
        }
        .map_err(SearchError::TraversalModelFailure)?;

        let (traversal_cost, result_state) = m
            .traversal_cost(&edge, &access_state)
            .map_err(SearchError::TraversalModelFailure)?;

        let result = EdgeTraversal {
            access_cost,
            traversal_cost,
            result_state,
        };

        Ok(result)
    }
}
