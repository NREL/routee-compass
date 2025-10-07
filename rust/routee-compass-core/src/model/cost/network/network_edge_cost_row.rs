use crate::model::{network::EdgeId, unit::Cost};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct NetworkEdgeCostRow {
    pub edge_id: EdgeId,
    pub cost: Cost,
}
