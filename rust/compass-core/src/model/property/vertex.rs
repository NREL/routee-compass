use geo::{coord, Coord};
use serde::Deserialize;

use rstar::{PointDistance, RTreeObject, AABB};

use crate::model::graph::vertex_id::VertexId;

#[derive(Copy, Clone, Deserialize, Default)]
pub struct Vertex {
    pub vertex_id: VertexId,
    pub coordinate: Coord<f32>,
}

impl Vertex {
    pub fn new(vertex_id: usize, x: f32, y: f32) -> Self {
        Self {
            vertex_id: VertexId(vertex_id),
            coordinate: coord! {x: x, y: y},
        }
    }
    pub fn to_tuple_underlying(&self) -> (f32, f32) {
        (self.coordinate.x, self.coordinate.y)
    }

    pub fn x(&self) -> f32 {
        self.coordinate.x
    }

    pub fn y(&self) -> f32 {
        self.coordinate.y
    }
}

impl RTreeObject for Vertex {
    type Envelope = AABB<Coord>;

    fn envelope(&self) -> Self::Envelope {
        AABB::from_corners(
            coord! {x: self.x() as f64, y: self.y() as f64},
            coord! {x: self.x() as f64, y: self.y() as f64},
        )
    }
}

impl PointDistance for Vertex {
    fn distance_2(&self, point: &Coord) -> f64 {
        let dx = self.x() as f64 - point.x;
        let dy = self.y() as f64 - point.y;
        dx * dx + dy * dy
    }
}
