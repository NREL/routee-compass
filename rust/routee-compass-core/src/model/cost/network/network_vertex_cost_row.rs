use crate::model::{network::VertexId, unit::Cost};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct NetworkVertexCostRow {
    pub vertex_id: VertexId,
    pub cost: Cost,
}
