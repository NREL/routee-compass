use super::{edge_id::EdgeId, vertex_id::VertexId};

#[derive(thiserror::Error, Debug, Clone)]
pub enum GraphError {
    #[error("edge {edge_id} not found")]
    EdgeIdNotFound { edge_id: EdgeId },
    #[error("edge attribute not found for edge {edge_id}")]
    EdgeAttributeNotFound { edge_id: EdgeId },
    #[error("vertex {vertex_id} not found")]
    VertexIdNotFound { vertex_id: VertexId },
    #[error("vertex attribute not found for vertex {vertex_id}")]
    VertexAttributeNotFound { vertex_id: VertexId },
    #[error("vertex without out edges in graph")]
    VertexWithoutOutEdges { vertex_id: VertexId },
    #[error("vertex without in edges in graph")]
    VertexWithoutInEdges { vertex_id: VertexId },
    #[error("error in test setup")]
    TestError,
}
