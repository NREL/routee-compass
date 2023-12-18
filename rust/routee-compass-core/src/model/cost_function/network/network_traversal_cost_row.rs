use crate::model::{cost_function::cost::Cost, road_network::edge_id::EdgeId};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct NetworkTraversalCostRow {
    pub edge_id: EdgeId,
    pub cost: Cost,
}
