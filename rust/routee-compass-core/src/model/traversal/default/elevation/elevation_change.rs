use std::borrow::Cow;

use crate::model::{
    state::{StateModel, StateModelError, StateVariable},
    traversal::default::fieldname,
    unit::{AsF64, Convert, Distance, DistanceUnit, Grade, GradeUnit, UnitError},
};

#[derive(Clone, Debug)]
pub struct ElevationChange {
    /// the change in elevation, positive or negative
    elevation: Distance,
    /// distance unit of the elevation change
    distance_unit: DistanceUnit,
}

impl ElevationChange {
    /// convert some distance and grade to an elevation change
    pub fn new(
        distance: (&Distance, &DistanceUnit),
        grade: (&Grade, &GradeUnit),
    ) -> Result<ElevationChange, UnitError> {
        let (d, du) = distance;
        let (g, gu) = grade;
        let mut g_dec = Cow::Borrowed(g);
        gu.convert(&mut g_dec, &GradeUnit::Decimal)?;

        let elevation = Distance::from(g_dec.as_f64() * d.as_f64());
        // let mut elevation = Cow::Owned(Distance::from(elevation_f64));
        // du.convert(&mut elevation, elevation_unit)?;

        Ok(ElevationChange {
            elevation,
            distance_unit: *du,
        })
    }

    /// adds this elevation change to the state vector. short circuits if elevation change is zero.
    /// updates using the following rules:
    ///
    /// - if self.elevation is positive:
    ///   - TRIP_ELEVATION_GAIN is incremented by self.elevation
    ///   - TRIP_ELEVATION_LOSS is unchanged
    /// - if self.elevation is negative:
    ///  - TRIP_ELEVATION_GAIN is unchanged
    ///  - TRIP_ELEVATION_LOSS is incremented by self.elevation
    pub fn add_elevation_to_state(
        &self,
        state: &mut [StateVariable],
        state_model: &StateModel,
    ) -> Result<(), StateModelError> {
        if self.elevation == Distance::ZERO {
            return Ok(());
        }
        let feature_name = if self.elevation < Distance::ZERO {
            fieldname::TRIP_ELEVATION_LOSS
        } else {
            fieldname::TRIP_ELEVATION_GAIN
        };
        state_model.add_distance(state, feature_name, &self.elevation, &self.distance_unit)
    }
}
