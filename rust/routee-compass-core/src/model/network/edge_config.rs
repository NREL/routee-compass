use crate::model::network::{edge_id::EdgeId, vertex_id::VertexId, Edge, EdgeListId};
use serde::{Deserialize, Serialize};

/// represents a single row in an edge list file.
#[derive(Copy, Clone, Serialize, Deserialize, Debug)]
pub struct EdgeConfig {
    pub edge_id: EdgeId,
    pub src_vertex_id: VertexId,
    pub dst_vertex_id: VertexId,
    /// assumed meters length of this edge
    pub distance: f64,
}

impl EdgeConfig {
    /// appends an EdgeListId to an EdgeConfig, translating it into a complete [`Edge`].
    pub fn assign_edge_list(&self, edge_list_id: &EdgeListId) -> Edge {
        Edge {
            edge_list_id: *edge_list_id,
            edge_id: self.edge_id,
            src_vertex_id: self.src_vertex_id,
            dst_vertex_id: self.dst_vertex_id,
            distance: uom::si::f64::Length::new::<uom::si::length::meter>(self.distance),
        }
    }
}
