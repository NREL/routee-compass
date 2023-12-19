use crate::model::{road_network::edge_id::EdgeId, utility::cost::Cost};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct NetworkTraversalCostRow {
    pub edge_id: EdgeId,
    pub cost: Cost,
}
