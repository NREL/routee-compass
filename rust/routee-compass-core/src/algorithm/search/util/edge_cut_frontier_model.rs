use std::{collections::HashSet, sync::Arc};

use crate::model::{frontier::frontier_model::FrontierModel, road_network::edge_id::EdgeId};

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
        edge: &crate::model::property::edge::Edge,
        state: &[crate::model::traversal::state::state_variable::StateVar],
        previous_edge: Option<&crate::model::property::edge::Edge>,
        state_model: &crate::model::state::state_model::StateModel,
    ) -> Result<bool, crate::model::frontier::frontier_model_error::FrontierModelError> {
        if self.cut_edges.contains(&edge.edge_id) {
            Ok(false)
        } else {
            self.underlying
                .valid_frontier(edge, state, previous_edge, state_model)
        }
    }
}
