use std::{cmp::Ordering, hash::Hash, hash::Hasher};

use crate::model::{cost::cost::Cost, graph::edge_id::EdgeId, graph::vertex_id::VertexId};

#[derive(Clone, Eq, PartialEq)]
pub struct VertexFrontier<S> {
    pub vertex_id: VertexId,
    pub prev_edge_id: Option<EdgeId>,
    pub state: S,
    pub cost: Cost,
}

impl<S: Eq> Hash for VertexFrontier<S> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.vertex_id.hash(state);
    }
}

impl<S: Eq> Ord for VertexFrontier<S> {
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

impl<S: Eq> PartialOrd for VertexFrontier<S> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
