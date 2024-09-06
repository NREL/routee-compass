use crate::model::road_network::{edge_id::EdgeId, vertex_id::VertexId};

/// simple 'Either' return type that covers both vertex-oriented and edge-oriented
/// Rtree implementations.
pub enum NearestSearchResult {
    NearestVertex(VertexId),
    NearestEdge(EdgeId),
}
