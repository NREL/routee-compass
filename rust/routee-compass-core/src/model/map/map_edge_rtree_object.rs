use super::{map_error::MapError, spatial_index_ops as ops};
use crate::model::network::{Edge, EdgeId};
use geo::{LineString, Point};
use rstar::{PointDistance, RTreeObject, AABB};
use uom::si::f64::Length;
use std::f32;
use crate::util::geo::haversine;

/// rtree element for edge-oriented map matching.
#[derive(Clone)]
pub struct MapEdgeRTreeObject {
    pub edge_id: EdgeId,
    pub envelope: AABB<Point<f32>>,
    // added line_coords variable
    pub line_coords: LineString<f32>,
}

impl MapEdgeRTreeObject {
    pub fn new(edge: &Edge, linestring: &LineString<f32>) -> MapEdgeRTreeObject {
        MapEdgeRTreeObject {
            edge_id: edge.edge_id,
            envelope: linestring.envelope(),
            // added line_coords
            line_coords: linestring.clone(),
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

impl RTreeObject for MapEdgeRTreeObject {
    type Envelope = AABB<Point<f32>>;

    fn envelope(&self) -> Self::Envelope {
        self.envelope
    }
}

impl PointDistance for MapEdgeRTreeObject {
    fn distance_2(&self, point: &Point<f32>) -> f32 {
        // get edge center point, linestring<f32>, line of coordinates from start and end of edge
        let coord_vec = self.line_coords.clone().into_inner();
        let mid_index = coord_vec.len()/2;
        let midpoint = coord_vec[mid_index];

        // use haversine to calculate distance
        match haversine::coord_distance(&midpoint.as_ref(), point.as_ref()){
            Ok(length) => length.value.powi(2) as f32,
            Err(error) => {println!("Error, invalid distance {error} "); f32::MAX}
        } 
    }
}
