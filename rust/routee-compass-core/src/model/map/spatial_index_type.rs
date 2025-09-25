//! configuration value to declare the type of [`super::SpatialIndex`] to build.
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub enum SpatialIndexType {
    #[default]
    #[serde(rename = "vertex")]
    VertexOriented,
    #[serde(rename = "edge")]
    EdgeOriented,
}
