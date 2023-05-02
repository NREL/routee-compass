use super::edge_id::EdgeId;
use super::vertex_id::VertexId;
use im::Vector;

pub trait DirectedGraph {
    fn out_edges(src: VertexId) -> Result<Vector<EdgeId>, String>;
    fn in_edges(src: VertexId) -> Result<Vector<EdgeId>, String>;
    fn src_vertex(edge_id: EdgeId) -> Result<VertexId, String>;
    fn dst_vertex(edge_id: EdgeId) -> Result<VertexId, String>;
}
