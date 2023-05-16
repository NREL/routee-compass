use std::cmp::Ordering;

use crate::model::{cost::cost::Cost, graph::edge_id::EdgeId};

#[derive(Clone, Eq, PartialEq)]
pub struct EdgeFrontier<S> {
    pub edge_id: EdgeId,
    pub prev_edge_id: Option<EdgeId>,
    pub state: S,
    pub cost: Cost,
}

impl<S: Eq> Ord for EdgeFrontier<S> {
    ///
    /// provides a min-ordering over Frontier costs
    /// is min-ordered due to order of comparitor (other.cmp(self))
    ///
    fn cmp(&self, other: &Self) -> Ordering {
        other
            .cost
            .cmp(&self.cost)
            .then_with(|| self.edge_id.cmp(&other.edge_id))
    }
}

impl<S: Eq> PartialOrd for EdgeFrontier<S> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
