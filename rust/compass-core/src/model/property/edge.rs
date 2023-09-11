use serde::{Deserialize, Serialize};

use uom::si;

use crate::model::graphv2::{edge_id::EdgeId, vertex_id::VertexId};

use crate::model::units::{Length, Ratio};

use super::road_class::RoadClass;

#[derive(Copy, Clone, Serialize, Deserialize, Debug)]
pub struct Edge {
    pub edge_id: EdgeId,
    pub src_vertex_id: VertexId,
    pub dst_vertex_id: VertexId,
    pub road_class: RoadClass,
    pub distance: Length,
    pub grade: Ratio,
}

impl Default for Edge {
    fn default() -> Self {
        Edge {
            edge_id: EdgeId(0),
            src_vertex_id: VertexId(0),
            dst_vertex_id: VertexId(1),
            road_class: RoadClass(1),
            distance: Length::new::<si::length::kilometer>(1.0),
            grade: Ratio::new::<si::ratio::percent>(0.0),
        }
    }
}
