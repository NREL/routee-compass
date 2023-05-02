use crate::model::property::edge::Edge;
use crate::model::property::vertex::Vertex;

use super::edge_id::EdgeId;
use super::vertex_id::VertexId;
use im::Vector;

pub trait DirectedGraph {
    fn edge_attr(&self, edge_id: EdgeId) -> Result<Edge, String>;
    fn vertex_attr(&self, vertex_id: VertexId) -> Result<Vertex, String>;
    fn out_edges(&self, src: VertexId) -> Result<Vector<EdgeId>, String>;
    fn in_edges(&self, src: VertexId) -> Result<Vector<EdgeId>, String>;
    fn src_vertex(&self, edge_id: EdgeId) -> Result<VertexId, String>;
    fn dst_vertex(&self, edge_id: EdgeId) -> Result<VertexId, String>;
}
