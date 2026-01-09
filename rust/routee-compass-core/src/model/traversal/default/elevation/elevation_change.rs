use uom::{
    si::f64::{Length, Ratio},
    ConstZero,
};

use crate::model::{
    state::{StateModel, StateModelError, StateVariable},
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
        trip_elevation_gain_idx: usize,
        trip_elevation_loss_idx: usize,
    ) -> Result<(), StateModelError> {
        if self.elevation == Length::ZERO {
            return Ok(());
        }
        let feature_idx = if self.elevation < Length::ZERO {
            trip_elevation_loss_idx
        } else {
            trip_elevation_gain_idx
        };

        // Use index-based access for performance - resolve index on each call
        // since this is called from different contexts
        state_model.add_distance_by_index(state, feature_idx, &self.elevation)
    }
}
