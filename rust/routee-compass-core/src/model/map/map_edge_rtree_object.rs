use super::{map_error::MapError, spatial_index_ops as ops};
use crate::model::{
    network::{Edge, EdgeId},
    unit::{Distance, DistanceUnit},
};
use geo::{LineString, Point};
use rstar::{PointDistance, RTreeObject, AABB};

/// rtree element for edge-oriented map matching.
#[derive(Clone)]
pub struct MapEdgeRTreeObject {
    pub edge_id: EdgeId,
    pub envelope: AABB<Point<f32>>,
}

impl MapEdgeRTreeObject {
    pub fn new(edge: &Edge, linestring: &LineString<f32>) -> MapEdgeRTreeObject {
        MapEdgeRTreeObject {
            edge_id: edge.edge_id,
            envelope: linestring.envelope(),
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

impl RTreeObject for MapEdgeRTreeObject {
    type Envelope = AABB<Point<f32>>;

    fn envelope(&self) -> Self::Envelope {
        self.envelope
    }
}

impl PointDistance for MapEdgeRTreeObject {
    fn distance_2(&self, point: &Point<f32>) -> f32 {
        self.envelope.distance_2(point)
    }
}
