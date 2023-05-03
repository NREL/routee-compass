use std::cmp::Ordering;

use crate::model::{cost::cost::Cost, graph::edge_id::EdgeId};

#[derive(Eq, PartialEq)]
pub struct Frontier<S> {
    pub edge_id: EdgeId,
    pub state: S,
    pub cost: Cost,
}

impl<S: Eq> Ord for Frontier<S> {
    ///
    /// provides a min-ordering over Frontier costs
    ///
    fn cmp(&self, other: &Self) -> Ordering {
        other
            .cost
            .cmp(&self.cost)
            .then_with(|| self.edge_id.cmp(&other.edge_id))
    }
}

impl<S: Eq> PartialOrd for Frontier<S> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
