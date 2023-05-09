use crate::model::property::edge::Edge;
use crate::model::property::vertex::Vertex;

use super::edge_id::EdgeId;
use super::graph_error::GraphError;
use super::vertex_id::VertexId;

pub trait DirectedGraph {
    fn edge_attr(&self, edge_id: EdgeId) -> Result<Edge, GraphError>;
    fn vertex_attr(&self, vertex_id: VertexId) -> Result<Vertex, GraphError>;
    fn out_edges(&self, src: VertexId) -> Result<Vec<EdgeId>, GraphError>;
    fn in_edges(&self, src: VertexId) -> Result<Vec<EdgeId>, GraphError>;
    fn src_vertex(&self, edge_id: EdgeId) -> Result<VertexId, GraphError>;
    fn dst_vertex(&self, edge_id: EdgeId) -> Result<VertexId, GraphError>;
}
