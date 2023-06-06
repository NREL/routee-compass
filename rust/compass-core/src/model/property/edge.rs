use crate::model::{
    graph::{edge_id::EdgeId, vertex_id::VertexId},
    units::{centimeters::Centimeters, cm_per_second::CmPerSecond, millis::Millis},
};

use super::road_class::RoadClass;

#[derive(Copy, Clone)]
pub struct Edge {
    pub edge_id: EdgeId,
    pub start_vertex: VertexId,
    pub end_vertex: VertexId,
    pub road_class: RoadClass,
    pub free_flow_speed_seconds: CmPerSecond,
    pub distance_centimeters: Centimeters,
    pub grade_millis: Millis,
}
