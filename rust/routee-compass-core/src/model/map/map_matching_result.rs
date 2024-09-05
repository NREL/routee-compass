use crate::model::road_network::{edge_id::EdgeId, vertex_id::VertexId};

pub enum MapMatchingResult {
    NearestVertex(VertexId),
    NearestEdge(EdgeId),
}
