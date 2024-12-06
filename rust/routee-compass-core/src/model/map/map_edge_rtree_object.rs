use super::{map_error::MapError, spatial_index_ops::within_threshold};
use crate::model::{
    network::{Edge, EdgeId},
    unit::{Distance, DistanceUnit},
};
use geo::{LineString, Point};
use rstar::{PointDistance, RTreeObject, AABB};

impl MapEdgeRTreeObject {
    pub fn new(edge: &Edge, linestring: &LineString<f32>) -> MapEdgeRTreeObject {
        MapEdgeRTreeObject {
            edge_id: edge.edge_id,
            envelope: linestring.envelope(),
        }
    }

    pub fn within_distance_threshold(
        &self,
        point: &Point<f32>,
        tolerance: &Option<(Distance, DistanceUnit)>,
    ) -> Result<(), MapError> {
        match tolerance {
            Some((dist, unit)) => within_threshold(&self.envelope, point, *dist, *unit),
            None => Ok(()),
        }
    }
}

#[derive(Clone)]
pub struct MapEdgeRTreeObject {
    pub edge_id: EdgeId,
    pub envelope: AABB<Point<f32>>,
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