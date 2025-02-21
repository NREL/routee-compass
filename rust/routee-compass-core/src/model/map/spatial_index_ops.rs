use std::borrow::Cow;

use super::map_error::MapError;
use crate::{
    model::unit::{Convert, Distance, DistanceUnit},
    util::geo::haversine,
};
use geo::Point;
use rstar::AABB;

pub fn test_threshold(
    envelope: &AABB<Point<f32>>,
    other: &Point<f32>,
    tolerance_distance: Distance,
    tolerance_distance_unit: DistanceUnit,
) -> Result<bool, MapError> {
    let this_coord = envelope.lower().0;
    let mut distance_meters = Cow::Owned(
        haversine::coord_distance_meters(&this_coord, &other.0).map_err(MapError::MapMatchError)?,
    );
    DistanceUnit::Meters.convert(&mut distance_meters, &tolerance_distance_unit)?;
    Ok(distance_meters.into_owned() >= tolerance_distance)
}

pub fn within_threshold(
    envelope: &AABB<Point<f32>>,
    other: &Point<f32>,
    tolerance_distance: Distance,
    tolerance_distance_unit: DistanceUnit,
) -> Result<(), MapError> {
    let this_coord = envelope.lower().0;
    let mut this_distance = Cow::Owned(
        haversine::coord_distance_meters(&this_coord, &other.0).map_err(MapError::MapMatchError)?,
    );
    DistanceUnit::Meters.convert(&mut this_distance, &tolerance_distance_unit)?;
    let this_distance_converted = this_distance.into_owned();
    if this_distance_converted >= tolerance_distance {
        Err(MapError::MapMatchError(
            format!(
                "coord {:?} nearest vertex coord is {:?} which is {} {} away, exceeding the distance tolerance of {} {}", 
                this_coord,
                other,
                this_distance_converted,
                tolerance_distance_unit,
                tolerance_distance,
                tolerance_distance_unit,
            )
        ))
    } else {
        Ok(())
    }
}
