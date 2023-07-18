use geo::{coord, Coord};
use serde::Deserialize;

use rstar::{PointDistance, RTreeObject, AABB};

use crate::model::{graph::vertex_id::VertexId, units::ordinate::Ordinate};

#[derive(Copy, Clone, Deserialize, Default)]
pub struct Vertex {
    pub vertex_id: VertexId,
    pub x: Ordinate,
    pub y: Ordinate,
}

impl Vertex {
    pub fn new(vertex_id: usize, x: f64, y: f64) -> Self {
        Self {
            vertex_id: VertexId(vertex_id),
            x: Ordinate(x),
            y: Ordinate(y),
        }
    }
    pub fn to_tuple_underlying(&self) -> (f64, f64) {
        (self.x.0, self.y.0)
    }
}

impl RTreeObject for Vertex {
    type Envelope = AABB<Coord>;

    fn envelope(&self) -> Self::Envelope {
        AABB::from_corners(
            coord! {x: self.x.0, y: self.y.0},
            coord! {x: self.x.0, y: self.y.0},
        )
    }
}

impl PointDistance for Vertex {
    fn distance_2(&self, point: &Coord) -> f64 {
        let dx = self.x.0 - point.x;
        let dy = self.y.0 - point.y;
        dx * dx + dy * dy
    }
}
