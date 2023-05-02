use im::Vector;

use crate::model::graph::edge_id::EdgeId;

pub struct Solution {
    edge_id: EdgeId,
    access_edge_id: Option<EdgeId>,
    access_cost: Vector<f64>,
}
