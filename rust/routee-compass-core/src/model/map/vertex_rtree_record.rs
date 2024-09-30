use crate::{
    model::{
        property::vertex::Vertex,
        unit::{Distance, DistanceUnit},
    },
    util::geo::haversine,
};
use geo::{coord, Coord};
use rstar::{PointDistance, RTreeObject, AABB};

use super::map_error::MapError;

/// representation of a graph vertex within the Rtree.
/// this record is a wrapper around the graph Vertex but does not hold a copy,
/// instead holds a reference (and lifetime) to reduce copying.
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

    /// confirms that this vertex is within some stated distance tolerance of a point.
    /// if no tolerance is provided, the dst coordinate is assumed to be a valid distance.
    ///
    /// # Arguments
    ///
    /// * `src` - source coordinate
    /// * `dst` - destination coordinate that may or may not be within some distance
    ///           tolerance of the src coordinate
    /// * `tolerance` - tolerance parameters set by user for the rtree plugin. if this is None,
    ///                 all coordinate pairs are assumed to be within distance tolerance, but this
    ///                 may lead to unexpected behavior where far away coordinates are considered "matched".
    ///
    /// # Returns
    ///
    /// * nothing, or an error if the coordinates are not within tolerance
    pub fn within_distance_threshold(
        self,
        other_coord: &Coord<f32>,
        tolerance: &Option<(Distance, DistanceUnit)>,
    ) -> Result<(), MapError> {
        match tolerance {
            Some((tolerance_distance, tolerance_distance_unit)) => {
                let this_coord = self.vertex.coordinate.0;
                let distance_meters = haversine::coord_distance_meters(&this_coord, other_coord)
                    .map_err(MapError::MapMatchError)?;
                let distance =
                    DistanceUnit::Meters.convert(&distance_meters, tolerance_distance_unit);
                if &distance >= tolerance_distance {
                    Err(MapError::MapMatchError(
                        format!(
                            "coord {:?} nearest vertex coord is {:?} which is {} {} away, exceeding the distance tolerance of {} {}", 
                            this_coord,
                            other_coord,
                            distance,
                            tolerance_distance_unit,
                            tolerance_distance,
                            tolerance_distance_unit,
                        )
                    ))
                } else {
                    Ok(())
                }
            }
            None => Ok(()),
        }
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
