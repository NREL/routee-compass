use std::cmp::Ordering;

use im::Vector;

use crate::model::{
    cost::{cost::Cost, metric::Metric},
    graph::edge_id::EdgeId,
};

#[derive(Clone, Eq, PartialEq)]
pub struct Frontier {
    pub edge_id: EdgeId,
    pub traverse_edge_metrics: Vector<Metric>,
    pub edge_edge_metrics: Vector<Metric>,
    pub cost: Cost,
}

impl Ord for Frontier {
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

impl PartialOrd for Frontier {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
