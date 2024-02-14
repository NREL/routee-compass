use std::hash::{Hash, Hasher};

use allocative::Allocative;

use crate::model::{road_network::edge_id::EdgeId, road_network::vertex_id::VertexId};

#[derive(Clone, Debug, Allocative)]
pub struct AStarFrontier {
    pub vertex_id: VertexId,
    pub prev_edge_id: Option<EdgeId>,
}

impl PartialEq for AStarFrontier {
    fn eq(&self, other: &Self) -> bool {
        self.vertex_id == other.vertex_id
    }
}
impl Eq for AStarFrontier {}

impl Hash for AStarFrontier {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.vertex_id.hash(state);
    }
}
