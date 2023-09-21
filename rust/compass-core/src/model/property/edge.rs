use super::road_class::RoadClass;
use crate::model::graph::{edge_id::EdgeId, vertex_id::VertexId};
use crate::util::unit::Distance;
use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Serialize, Deserialize, Debug)]
pub struct Edge {
    pub edge_id: EdgeId,
    pub src_vertex_id: VertexId,
    pub dst_vertex_id: VertexId,
    pub road_class: RoadClass,
    pub distance: Distance,
    pub grade: f64,
}

impl Default for Edge {
    fn default() -> Self {
        Edge {
            edge_id: EdgeId(0),
            src_vertex_id: VertexId(0),
            dst_vertex_id: VertexId(1),
            road_class: RoadClass(1),
            distance: Distance::ONE,
            grade: 0.0,
        }
    }
}
