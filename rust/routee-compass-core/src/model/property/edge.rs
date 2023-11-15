use crate::model::road_network::{edge_id::EdgeId, vertex_id::VertexId};
use crate::util::unit::Distance;
use serde::{Deserialize, Serialize};

/// represents a single edge in a Graph.
/// this struct implements Serialize and Deserialize to support reading
/// edge records from CSV files.
#[derive(Copy, Clone, Serialize, Deserialize, Debug)]
pub struct Edge {
    pub edge_id: EdgeId,
    pub src_vertex_id: VertexId,
    pub dst_vertex_id: VertexId,
    pub distance: Distance,
}

impl Edge {
    pub fn new(edge_id: usize, src_vertex_id: usize, dst_vertex_id: usize, distance: f64) -> Self {
        Self {
            edge_id: EdgeId(edge_id),
            src_vertex_id: VertexId(src_vertex_id),
            dst_vertex_id: VertexId(dst_vertex_id),
            distance: Distance::new(distance),
        }
    }
}

impl Default for Edge {
    fn default() -> Self {
        Edge {
            edge_id: EdgeId(0),
            src_vertex_id: VertexId(0),
            dst_vertex_id: VertexId(1),
            distance: Distance::ONE,
        }
    }
}
