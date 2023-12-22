use crate::model::{road_network::edge_id::EdgeId, unit::Cost};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct NetworkTraversalUtilityRow {
    pub edge_id: EdgeId,
    pub cost: Cost,
}
