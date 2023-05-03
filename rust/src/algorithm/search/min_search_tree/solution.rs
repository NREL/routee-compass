use im::Vector;

use crate::model::{cost::cost::Cost, graph::edge_id::EdgeId};

pub struct Solution {
    edge_id: EdgeId,
    access_edge_id: Option<EdgeId>,
    access_cost: Cost,
    traversal_cost: Cost,
    total_cost: Cost,
}
