use crate::model::{
    frontier::FrontierModel,
    network::{Edge, EdgeId},
};
use std::{collections::HashSet, sync::Arc};

/// A wrapper of the user-generated FrontierModel which prohibits traversals
/// on selected edges. algorithms can create this wrapper with a set of "cut edges"
/// and the search will not allow traversal of these edges.
pub struct EdgeCutFrontierModel {
    pub underlying: Arc<dyn FrontierModel>,
    cut_edges: HashSet<EdgeId>,
}

impl EdgeCutFrontierModel {
    pub fn new(
        underlying: Arc<dyn FrontierModel>,
        cut_edges: HashSet<EdgeId>,
    ) -> EdgeCutFrontierModel {
        EdgeCutFrontierModel {
            underlying,
            cut_edges,
        }
    }
}

impl FrontierModel for EdgeCutFrontierModel {
    fn valid_frontier(
        &self,
        edge: &Edge,
        state: &[crate::model::state::StateVariable],
        tree: &std::collections::HashMap<
            crate::model::network::VertexId,
            crate::algorithm::search::SearchTreeBranch,
        >,
        direction: &crate::algorithm::search::Direction,
        state_model: &crate::model::state::StateModel,
    ) -> Result<bool, crate::model::frontier::FrontierModelError> {
        if self.cut_edges.contains(&edge.edge_id) {
            Ok(false)
        } else {
            self.underlying
                .valid_frontier(edge, state, tree, direction, state_model)
        }
    }

    fn valid_edge(&self, edge: &Edge) -> Result<bool, crate::model::frontier::FrontierModelError> {
        if self.cut_edges.contains(&edge.edge_id) {
            self.underlying.valid_edge(edge)
        } else {
            Ok(false)
        }
    }
}
