use super::map_input_type::MapInputType;
use crate::model::unit::{Distance, DistanceUnit};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(tag = "type")]
pub enum MapModelConfig {
    #[serde(rename = "vertex")]
    VertexMapModelConfig {
        map_input_type: MapInputType,
        tolerance: Option<(Distance, DistanceUnit)>,
        geometry_input_file: Option<String>,
        queries_without_destinations: bool,
    },
    #[serde(rename = "edge")]
    EdgeMapModelConfig {
        map_input_type: MapInputType,
        tolerance: Option<(Distance, DistanceUnit)>,
        geometry_input_file: String,
        queries_without_destinations: bool,
    },
}
