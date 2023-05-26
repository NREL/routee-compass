use crate::model::{
    graph::vertex_id::VertexId,
    units::{centimeters::Centimeters, cm_per_second::CmPerSecond, millis::Millis, seconds::Seconds},
};

use super::road_class::RoadClass;

#[derive(Copy, Clone)]
pub struct Edge {
    pub start_vertex: VertexId,
    pub end_vertex: VertexId,
    pub road_class: RoadClass,
    pub free_flow_speed_cm_per_second: CmPerSecond,
    pub distance_centimeters: Centimeters,
    pub grade_millis: Millis,
}

impl Edge {
    pub fn free_flow_travel_time_seconds(&self) -> Seconds {
        Seconds(self.distance_centimeters.0 as i64 / self.free_flow_speed_cm_per_second.0 as i64)
    }
}
