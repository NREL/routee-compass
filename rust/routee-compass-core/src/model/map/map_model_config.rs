use crate::model::unit::{Distance, DistanceUnit};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum MapModelConfig {
    VertexMapModelConfig {
        tolerance: Option<(Distance, DistanceUnit)>,
        geometry_input_file: Option<String>,
    },
    EdgeMapModelConfig {
        tolerance: Option<(Distance, DistanceUnit)>,
        geometry_input_file: String,
    },
}
