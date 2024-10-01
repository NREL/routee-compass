use super::map_error::MapError;
use crate::model::{
    property::edge::Edge,
    unit::{Distance, DistanceUnit},
};
use geo::{Centroid, ClosestPoint, Coord, Geometry, LineString, Point};
use proj::Transform;
use rstar::{PointDistance, RTreeObject, AABB};
use wkt::ToWkt;

#[derive(Clone, Copy, Debug)]
pub struct EdgeRtreeRecord<'a> {
    pub edge: &'a Edge,
    pub geometry: &'a LineString<f32>,
}

impl<'a> EdgeRtreeRecord<'a> {
    pub fn new(edge: &'a Edge, geometry: &'a LineString<f32>) -> EdgeRtreeRecord<'a> {
        EdgeRtreeRecord { edge, geometry }
    }

    /// confirms that this vertex is within some stated distance tolerance of a point.
    /// if no tolerance is provided, the dst coordinate is assumed to be a valid distance.
    ///
    /// # Arguments
    ///
    /// * `coord`     - coordinate to check distance to this edge
    /// * `tolerance` - tolerance parameters set by user for the rtree plugin. if this is None,
    ///                 all coordinate pairs are assumed to be within distance tolerance, but this
    ///                 may lead to unexpected behavior where far away coordinates are considered "matched".
    ///
    /// # Returns
    ///
    /// * nothing, or an error if the coordinates are not within tolerance
    pub fn within_distance_threshold(
        self,
        coord: &Coord<f32>,
        tolerance: &Option<(Distance, DistanceUnit)>,
    ) -> Result<(), MapError> {
        match tolerance {
            Some((dist, unit)) => {
                let query_point = geo::Point(*coord);
                let line_merc: Geometry<f32> =
                    to_web_mercator(&geo::Geometry::LineString(self.geometry.clone()))?;
                let point_merc_geom: Geometry<f32> =
                    to_web_mercator(&geo::Geometry::Point(query_point))?;
                let point_merc = match point_merc_geom {
                    Geometry::Point(point) => Ok(point),
                    _ => Err(MapError::InternalError(String::from(
                        "Point changed into something else!",
                    ))),
                }?;

                let distance = match line_merc.closest_point(&point_merc) {
                    geo::Closest::Intersection(closest_point) => {
                        Ok(closest_point.distance_2(&query_point))
                    }
                    geo::Closest::SinglePoint(closest_point) => {
                        Ok(closest_point.distance_2(&query_point))
                    }
                    geo::Closest::Indeterminate => Err(MapError::DistanceThresholdError(
                        String::from("closest point is indeterminate"),
                        *dist,
                        *unit,
                    )),
                }?;

                let dist_in_unit =
                    DistanceUnit::Meters.convert(&Distance::new(distance.into()), unit);
                if &dist_in_unit >= dist {
                    Err(MapError::MapMatchError(
                        format!(
                            "coord {:?} nearest edge is {} which is {} {} away, exceeding the distance tolerance of {}/{}", 
                            coord,
                            self.edge.edge_id,
                            distance,
                            unit,
                            dist,
                            unit,
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

impl<'a> RTreeObject for EdgeRtreeRecord<'a> {
    type Envelope = AABB<Point<f32>>;
    fn envelope(&self) -> Self::Envelope {
        self.geometry.envelope()
    }
}

impl<'a> PointDistance for EdgeRtreeRecord<'a> {
    /// compares query nearness via the "centroid" of this LineString,
    /// the midpoint of the bounding box of the line.
    ///
    /// # Arguments
    ///
    /// * `point` - point query of a nearest neighbors search
    ///
    /// # Returns
    ///
    /// * distance in meters (assumes points are in WGS84)
    fn distance_2(&self, point: &Point<f32>) -> f32 {
        let this_point = self
            .geometry
            .centroid()
            .unwrap_or_else(|| panic!("empty linestring in geometry file"));
        // as noted in the comments for PointDistance, this should return the squared distance.
        // haversine *should* work but squared haversine in meters is giving weird results for
        // the vertex rtree plugin, so, i'm reverting this to euclidean for now. -rjf 2023-12-01
        // let distance = haversine::coord_distance_meters(this_point.0, point.0)
        //     .unwrap_or(Distance::new(f64::MAX))
        //     .as_f64();
        // distance * distance
        let dx = this_point.x() - point.x();
        let dy = this_point.y() - point.y();
        dx * dx + dy * dy
    }
}

fn to_web_mercator(geometry: &Geometry<f32>) -> Result<Geometry<f32>, MapError> {
    let mut out_geom = geometry.clone();
    out_geom
        .transform_crs_to_crs("WGS84", "EPSG:3857")
        .map_err(|error| {
            let mut geom_string: String = geometry.to_wkt().to_string();
            geom_string.truncate(50);
            MapError::ProjectionError {
                geometry: geom_string,
                error: error.to_string(),
            }
        })?;
    Ok(out_geom)
}
