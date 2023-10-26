use crate::model::graph::{edge_id::EdgeId, vertex_id::VertexId};
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
