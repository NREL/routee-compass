use crate::model::network::{edge_id::EdgeId, vertex_id::VertexId, EdgeListId};
use serde::{Deserialize, Serialize};

use uom::si::f64::Length;

/// represents a single edge in a Graph.
/// this struct implements Serialize and Deserialize to support reading
/// edge records from CSV files.
#[derive(Copy, Clone, Serialize, Deserialize, Debug)]
pub struct Edge {
    pub edge_list_id: EdgeListId,
    pub edge_id: EdgeId,
    pub src_vertex_id: VertexId,
    pub dst_vertex_id: VertexId,
    pub distance: Length,
}

impl Edge {
    pub fn new(
        edge_list_id: usize,
        edge_id: usize,
        src_vertex_id: usize,
        dst_vertex_id: usize,
        distance: Length,
    ) -> Self {
        Self {
            edge_list_id: EdgeListId(edge_list_id),
            edge_id: EdgeId(edge_id),
            src_vertex_id: VertexId(src_vertex_id),
            dst_vertex_id: VertexId(dst_vertex_id),
            distance,
        }
    }
}
