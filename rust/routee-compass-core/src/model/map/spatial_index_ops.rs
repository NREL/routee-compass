use super::map_error::MapError;
use crate::{
    model::unit::{Distance, DistanceUnit},
    util::geo::haversine,
};
use geo::Point;
use rstar::AABB;

pub fn within_threshold(
    envelope: &AABB<Point<f32>>,
    other: &Point<f32>,
    tolerance_distance: Distance,
    tolerance_distance_unit: DistanceUnit,
) -> Result<(), MapError> {
    let this_coord = envelope.lower().0;
    let distance_meters =
        haversine::coord_distance_meters(&this_coord, &other.0).map_err(MapError::MapMatchError)?;
    let distance = DistanceUnit::Meters.convert(&distance_meters, &tolerance_distance_unit);
    if distance >= tolerance_distance {
        Err(MapError::MapMatchError(
            format!(
                "coord {:?} nearest vertex coord is {:?} which is {} {} away, exceeding the distance tolerance of {} {}", 
                this_coord,
                other,
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
