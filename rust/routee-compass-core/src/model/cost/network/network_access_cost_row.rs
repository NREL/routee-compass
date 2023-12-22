use crate::model::{road_network::edge_id::EdgeId, unit::Cost};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct NetworkAccessUtilityRow {
    pub source: EdgeId,
    pub destination: EdgeId,
    pub cost: Cost,
}
