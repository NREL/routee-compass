use crate::model::traversal::state::search_state::SearchState;
use crate::model::{cost::cost::Cost, graph::edge_id::EdgeId};
use std::cmp::Ordering;

#[derive(Clone)]
pub struct EdgeFrontier {
    pub edge_id: EdgeId,
    pub prev_edge_id: Option<EdgeId>,
    pub state: SearchState,
    pub cost: Cost,
}

impl PartialEq for EdgeFrontier {
    fn eq(&self, other: &Self) -> bool {
        self.edge_id == other.edge_id
    }
}

impl Eq for EdgeFrontier {}

impl Ord for EdgeFrontier {
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

impl PartialOrd for EdgeFrontier {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
