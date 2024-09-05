use crate::model::property::vertex::Vertex;
use geo::{coord, Coord};
use rstar::{PointDistance, RTreeObject, AABB};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy)]
pub struct VertexRTreeRecord<'a> {
    pub vertex: &'a Vertex,
}

impl<'a> VertexRTreeRecord<'a> {
    pub fn new(vertex: &'a Vertex) -> Self {
        Self { vertex }
    }
    pub fn x(&self) -> f32 {
        self.vertex.x()
    }
    pub fn y(&self) -> f32 {
        self.vertex.y()
    }
}

impl<'a> RTreeObject for VertexRTreeRecord<'a> {
    type Envelope = AABB<Coord<f32>>;

    fn envelope(&self) -> Self::Envelope {
        AABB::from_corners(
            coord! {x: self.x(), y: self.y()},
            coord! {x: self.x(), y: self.y()},
        )
    }
}

impl<'a> PointDistance for VertexRTreeRecord<'a> {
    fn distance_2(&self, point: &Coord<f32>) -> f32 {
        let dx = self.x() - point.x;
        let dy = self.y() - point.y;
        dx * dx + dy * dy
    }
}
