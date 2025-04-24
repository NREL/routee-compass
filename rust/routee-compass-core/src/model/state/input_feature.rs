use serde::{Deserialize, Serialize};

use crate::model::unit::{DistanceUnit, EnergyUnit, GradeUnit, SpeedUnit, TimeUnit};

/// defines the required input feature and its requested unit type for a given state variable
///
/// if a unit type is provided, then the state variable is provided in the requested unit to the model.
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(tag = "type", content = "convert", rename_all = "snake_case")]
pub enum InputFeature {
    Distance(Option<DistanceUnit>),
    Speed(Option<SpeedUnit>),
    Time(Option<TimeUnit>),
    Energy(Option<EnergyUnit>),
    Grade(Option<GradeUnit>),
    Custom,
}
