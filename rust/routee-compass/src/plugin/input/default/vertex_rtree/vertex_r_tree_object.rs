use geo_types::Coord;
use routee_compass_core::{
    model::property::vertex::Vertex,
    util::{geo::haversine, unit::as_f64::AsF64},
};
use rstar::{PointDistance, RTreeObject, AABB};

pub struct VertexRTreeObject {
    pub vertex: Vertex,
}

impl VertexRTreeObject {
    pub fn new(vertex: Vertex) -> Self {
        Self { vertex }
    }
    pub fn x(&self) -> f64 {
        self.vertex.x()
    }
    pub fn y(&self) -> f64 {
        self.vertex.y()
    }
}

impl RTreeObject for VertexRTreeObject {
    type Envelope = AABB<Coord>;

    fn envelope(&self) -> Self::Envelope {
        AABB::from_point(self.vertex.coordinate)
    }
}

impl PointDistance for VertexRTreeObject {
    fn distance_2(&self, point: &Coord) -> f64 {
        haversine::coord_distance_meters(self.vertex.coordinate, *point)
            .unwrap()
            .as_f64()
    }
}
