use super::{map_error::MapError, spatial_index_ops as ops};
use crate::model::{
    network::{Vertex, VertexId},
    unit::{Distance, DistanceUnit},
};
use geo::{coord, Point};
use rstar::{PointDistance, RTreeObject, AABB};

/// rtree element for vertex-oriented map matching.
#[derive(Clone)]
pub struct MapVertexRTreeObject {
    pub vertex_id: VertexId,
    pub envelope: AABB<Point<f32>>,
}

impl PointDistance for MapVertexRTreeObject {
    fn distance_2(&self, point: &Point<f32>) -> f32 {
        self.envelope.distance_2(point)
    }
}

impl MapVertexRTreeObject {
    pub fn new(vertex: &Vertex) -> MapVertexRTreeObject {
        MapVertexRTreeObject {
            vertex_id: vertex.vertex_id,
            envelope: AABB::from_corners(
                geo::Point(coord! {x: vertex.x(), y: vertex.y()}),
                geo::Point(coord! {x: vertex.x(), y: vertex.y()}),
            ),
        }
    }

    pub fn test_threshold(
        &self,
        point: &Point<f32>,
        tolerance: &Option<(Distance, DistanceUnit)>,
    ) -> Result<bool, MapError> {
        match tolerance {
            Some((dist, unit)) => ops::test_threshold(&self.envelope, point, *dist, *unit),
            None => Ok(true),
        }
    }

    pub fn within_distance_threshold(
        &self,
        point: &Point<f32>,
        tolerance: &Option<(Distance, DistanceUnit)>,
    ) -> Result<(), MapError> {
        match tolerance {
            Some((dist, unit)) => ops::within_threshold(&self.envelope, point, *dist, *unit),
            None => Ok(()),
        }
    }
}

impl RTreeObject for MapVertexRTreeObject {
    type Envelope = AABB<Point<f32>>;

    fn envelope(&self) -> Self::Envelope {
        self.envelope
    }
}
