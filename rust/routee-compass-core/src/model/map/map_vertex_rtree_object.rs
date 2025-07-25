use super::{map_error::MapError, spatial_index_ops as ops};
use crate::model::network::{Vertex, VertexId};
use geo::{coord, Point};
use rstar::{PointDistance, RTreeObject, AABB};
use uom::si::f64::Length;
use crate::util::geo::haversine;
use rstar::Envelope;
use std::f32;

/// rtree element for vertex-oriented map matching.
#[derive(Clone)]
pub struct MapVertexRTreeObject {
    pub vertex_id: VertexId,
    pub envelope: AABB<Point<f32>>,
}

impl PointDistance for MapVertexRTreeObject {
    fn distance_2(&self, point: &Point<f32>) -> f32 {
        // use haversine to calculate distance
        match haversine::coord_distance(&self.envelope.center().as_ref(), point.as_ref()){
            Ok(length) => length.value.powi(2) as f32,
            Err(error) => {println!("Error, invalid distance {error} "); f32::MAX}
        } 
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
        tolerance: &Option<Length>,
    ) -> Result<bool, MapError> {
        match tolerance {
            Some(dist) => ops::test_threshold(&self.envelope, point, *dist),
            None => Ok(true),
        }
    }

    pub fn within_distance_threshold(
        &self,
        point: &Point<f32>,
        tolerance: &Option<Length>,
    ) -> Result<(), MapError> {
        match tolerance {
            Some(dist) => ops::within_threshold(&self.envelope, point, *dist),
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
