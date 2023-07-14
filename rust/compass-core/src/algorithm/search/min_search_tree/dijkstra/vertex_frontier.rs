use std::{cmp::Ordering, hash::Hash, hash::Hasher};

use crate::model::traversal::state::search_state::SearchState;

use crate::model::{cost::cost::Cost, graph::edge_id::EdgeId, graph::vertex_id::VertexId};

#[derive(Clone)]
pub struct VertexFrontier {
    pub vertex_id: VertexId,
    pub prev_edge_id: Option<EdgeId>,
    pub state: SearchState,
    pub cost: Cost,
}

impl PartialEq for VertexFrontier {
    fn eq(&self, other: &Self) -> bool {
        self.vertex_id == other.vertex_id
    }
}

impl Eq for VertexFrontier {}

impl Hash for VertexFrontier {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.vertex_id.hash(state);
    }
}

impl Ord for VertexFrontier {
    ///
    /// provides a min-ordering over Frontier costs
    /// is min-ordered due to order of comparitor (other.cmp(self))
    ///
    fn cmp(&self, other: &Self) -> Ordering {
        other
            .cost
            .cmp(&self.cost)
            .then_with(|| self.vertex_id.cmp(&other.vertex_id))
    }
}

impl PartialOrd for VertexFrontier {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
