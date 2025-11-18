use crate::model::{
    filter::FilterModel,
    network::{Edge, EdgeId},
};
use std::{collections::HashSet, sync::Arc};

/// A wrapper of the user-generated FilterModel which prohibits traversals
/// on selected edges. algorithms can create this wrapper with a set of "cut edges"
/// and the search will not allow traversal of these edges.
pub struct EdgeCutFilterModel {
    pub underlying: Arc<dyn FilterModel>,
    cut_edges: HashSet<EdgeId>,
}

impl EdgeCutFilterModel {
    pub fn new(underlying: Arc<dyn FilterModel>, cut_edges: HashSet<EdgeId>) -> EdgeCutFilterModel {
        EdgeCutFilterModel {
            underlying,
            cut_edges,
        }
    }
}

impl FilterModel for EdgeCutFilterModel {
    fn valid_frontier(
        &self,
        edge: &Edge,
        previous_edge: Option<&Edge>,
        state: &[crate::model::state::StateVariable],
        state_model: &crate::model::state::StateModel,
    ) -> Result<bool, crate::model::filter::FilterModelError> {
        if self.cut_edges.contains(&edge.edge_id) {
            Ok(false)
        } else {
            self.underlying
                .valid_frontier(edge, previous_edge, state, state_model)
        }
    }

    fn valid_edge(&self, edge: &Edge) -> Result<bool, crate::model::filter::FilterModelError> {
        if self.cut_edges.contains(&edge.edge_id) {
            self.underlying.valid_edge(edge)
        } else {
            Ok(false)
        }
    }
}
