use serde::{Deserialize, Serialize};

use crate::model::unit::{DistanceUnit, EnergyUnit, GradeUnit, SpeedUnit, TimeUnit};

use super::OutputFeature;

/// defines the required input feature and its requested unit type for a given state variable
///
/// if a unit type is provided, then the state variable is provided in the requested unit to the model.
#[derive(Serialize, Deserialize, Clone, Debug, Hash, Eq, PartialEq)]
#[serde(tag = "type", content = "convert", rename_all = "snake_case")]
pub enum InputFeature {
    Distance(DistanceUnit),
    Speed(SpeedUnit),
    Time(TimeUnit),
    Energy(EnergyUnit),
    Grade(GradeUnit),
    Custom { r#type: String, unit: String },
}

impl std::fmt::Display for InputFeature {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = serde_json::to_string_pretty(self).unwrap_or_default();
        write!(f, "{}", s)
    }
}

impl From<&OutputFeature> for InputFeature {
    fn from(value: &OutputFeature) -> Self {
        match value {
            OutputFeature::Distance {
                distance_unit,
                initial: _,
                accumulator: _,
            } => InputFeature::Distance(*distance_unit),
            OutputFeature::Time {
                time_unit,
                initial: _,
                accumulator: _,
            } => InputFeature::Time(*time_unit),
            OutputFeature::Energy {
                energy_unit,
                initial: _,
                accumulator: _,
            } => InputFeature::Energy(*energy_unit),
            OutputFeature::Speed {
                speed_unit,
                initial: _,
                accumulator: _,
            } => InputFeature::Speed(*speed_unit),
            OutputFeature::Grade {
                grade_unit,
                initial: _,
                accumulator: _,
            } => InputFeature::Grade(*grade_unit),
            OutputFeature::Custom {
                r#type,
                unit,
                format: _,
                accumulator: _,
            } => InputFeature::Custom {
                r#type: r#type.clone(),
                unit: unit.clone(),
            },
        }
    }
}
