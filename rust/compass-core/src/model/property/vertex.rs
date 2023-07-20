use geo::{coord, Coord};
use serde::Deserialize;

use rstar::{PointDistance, RTreeObject, AABB};

use crate::model::graph::vertex_id::VertexId;

#[derive(Copy, Clone, Deserialize, Default)]
pub struct Vertex {
    pub vertex_id: VertexId,
    pub coordinate: Coord,
}

impl Vertex {
    pub fn new(vertex_id: usize, x: f64, y: f64) -> Self {
        Self {
            vertex_id: VertexId(vertex_id),
            coordinate: coord! {x: x, y: y},
        }
    }
    pub fn to_tuple_underlying(&self) -> (f64, f64) {
        (self.coordinate.x, self.coordinate.y)
    }

    pub fn x(&self) -> f64 {
        self.coordinate.x
    }

    pub fn y(&self) -> f64 {
        self.coordinate.y
    }
}

impl RTreeObject for Vertex {
    type Envelope = AABB<Coord>;

    fn envelope(&self) -> Self::Envelope {
        AABB::from_corners(
            coord! {x: self.x(), y: self.y()},
            coord! {x: self.x(), y: self.y()},
        )
    }
}

impl PointDistance for Vertex {
    fn distance_2(&self, point: &Coord) -> f64 {
        let dx = self.x() - point.x;
        let dy = self.y() - point.y;
        dx * dx + dy * dy
    }
}
