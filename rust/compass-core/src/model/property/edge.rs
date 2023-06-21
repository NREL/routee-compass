use serde::Deserialize;

use crate::model::{
    graph::{edge_id::EdgeId, vertex_id::VertexId},
    units::{centimeters::Centimeters, cm_per_second::CmPerSecond, millis::Millis},
};

use super::road_class::RoadClass;

#[derive(Copy, Clone, Deserialize, Debug, Default)]
pub struct Edge {
    pub edge_id: EdgeId,
    pub src_vertex_id: VertexId,
    pub dst_vertex_id: VertexId,
    pub road_class: RoadClass,
    pub free_flow_speed_cps: CmPerSecond,
    pub distance_centimeters: Centimeters,
    pub grade_millis: Millis,
}
