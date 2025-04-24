use crate::model::unit::DistanceUnit;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct ElevationConfiguration {
    /// unit to record time
    pub distance_unit: DistanceUnit,
}
