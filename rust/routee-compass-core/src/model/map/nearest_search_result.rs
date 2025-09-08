use crate::model::network::{EdgeId, EdgeListId, VertexId};

/// simple 'Either' return type that covers both vertex-oriented and edge-oriented
/// search implementations.
pub enum NearestSearchResult {
    NearestVertex(VertexId),
    NearestEdge(EdgeListId, EdgeId),
}
