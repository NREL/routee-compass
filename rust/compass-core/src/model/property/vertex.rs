use serde::Deserialize;

use crate::model::{graph::vertex_id::VertexId, units::ordinate::Ordinate};

#[derive(Copy, Clone, Deserialize, Default)]
pub struct Vertex {
    pub vertex_id: VertexId,
    pub x: Ordinate,
    pub y: Ordinate,
}

impl Vertex {
    pub fn to_tuple_underlying(&self) -> (f64, f64) {
        (self.x.0, self.y.0)
    }
}
