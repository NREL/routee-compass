use uom::{
    si::f64::{Length, Ratio},
    ConstZero,
};

use crate::model::{
    state::{StateModel, StateModelError, StateVariable},
    traversal::default::fieldname,
    unit::UnitError,
};

#[derive(Clone, Debug)]
pub struct ElevationChange {
    /// the change in elevation, positive or negative
    elevation: Length,
}

impl ElevationChange {
    /// convert some distance and grade to an elevation change
    pub fn new(distance: Length, grade: Ratio) -> Result<ElevationChange, UnitError> {
        let elevation = distance * grade;

        Ok(ElevationChange { elevation })
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
        if self.elevation == Length::ZERO {
            return Ok(());
        }
        let feature_name = if self.elevation < Length::ZERO {
            fieldname::TRIP_ELEVATION_LOSS
        } else {
            fieldname::TRIP_ELEVATION_GAIN
        };
        state_model.add_distance(state, feature_name, &self.elevation)
    }
}
