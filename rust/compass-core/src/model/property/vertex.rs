use serde::Deserialize;

use crate::model::{graph::vertex_id::VertexId, units::ordinate::Ordinate};

#[derive(Copy, Clone, Deserialize, Default)]
pub struct Vertex {
    pub vertex_id: VertexId,
    pub x: Ordinate,
    pub y: Ordinate,
}
