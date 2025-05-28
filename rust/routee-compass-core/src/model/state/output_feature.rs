use super::{
    custom_feature_format::CustomFeatureFormat, state_model_error::StateModelError, InputFeature,
    StateVariable,
};
use crate::model::unit;
use serde::{Deserialize, Serialize};
use std::fmt::Display;

/// a state output feature tracks the domain of a StateVar in a
/// state vector. if the value represents quantity in one of the
/// unit types defined in crate::model::unit, then we have a system of internal unit
/// objects which provide conversion arithmetic. if the user
/// specifies a StateVar has a custom state variable unit, then
/// they provide a mapping codec and name for the variable, and
/// it does not interact with our native unit system.
///
/// # Example
///
/// ### Deserialization
///
/// an example TOML representation of state output features:
///
/// ```toml
/// state = [
///   { distance_unit = "kilometers", initial = 0.0 },
///   { time_unit = "minutes", initial = 0.0 },
///   { name = "soc", unit = "percent", format = { type = "floating_point", initial = 0.0 } }
/// ]
///
/// NOTE: deserialization is "untagged" so each variant must have a unique set of
/// field names. see link for more information:
/// https://serde.rs/enum-representations.html#untagged
/// ```
#[derive(Serialize, Deserialize, Clone, Debug, Eq)]
#[serde(untagged)]
pub enum OutputFeature {
    Distance {
        distance_unit: unit::DistanceUnit,
        initial: unit::Distance,
        accumulator: bool,
    },
    Time {
        time_unit: unit::TimeUnit,
        initial: unit::Time,
        accumulator: bool,
    },
    Energy {
        energy_unit: unit::EnergyUnit,
        initial: unit::Energy,
        accumulator: bool,
    },
    Speed {
        speed_unit: unit::SpeedUnit,
        initial: unit::Speed,
        accumulator: bool,
    },
    Grade {
        grade_unit: unit::GradeUnit,
        initial: unit::Grade,
        accumulator: bool,
    },
    Custom {
        name: String,
        unit: String,
        format: CustomFeatureFormat,
        accumulator: bool,
    },
}

impl PartialEq for OutputFeature {
    /// tests equality based on the feature type.
    ///
    /// for distance|time|energy, it's fine to modify either the unit
    /// or the initial value as this should not interfere with properly-
    /// implemented TraversalModel, AccessModel, and FrontierModel instances.
    ///
    /// for custom features, we are stricter about this equality test.
    /// for instance, we cannot allow a user to change the "meaning" of a
    /// state of charge value, that it is a floating point value in the range [0.0, 1.0].
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (
                OutputFeature::Distance {
                    distance_unit: _,
                    initial: _,
                    accumulator: _,
                },
                OutputFeature::Distance {
                    distance_unit: _,
                    initial: _,
                    accumulator: _,
                },
            ) => true,
            (
                OutputFeature::Time {
                    time_unit: _,
                    initial: _,
                    accumulator: _,
                },
                OutputFeature::Time {
                    time_unit: _,
                    initial: _,
                    accumulator: _,
                },
            ) => true,
            (
                OutputFeature::Energy {
                    energy_unit: _,
                    initial: _,
                    accumulator: _,
                },
                OutputFeature::Energy {
                    energy_unit: _,
                    initial: _,
                    accumulator: _,
                },
            ) => true,
            (
                OutputFeature::Speed {
                    speed_unit: _,
                    initial: _,
                    accumulator: _,
                },
                OutputFeature::Speed {
                    speed_unit: _,
                    initial: _,
                    accumulator: _,
                },
            ) => true,
            (
                OutputFeature::Grade {
                    grade_unit: _,
                    initial: _,
                    accumulator: _,
                },
                OutputFeature::Grade {
                    grade_unit: _,
                    initial: _,
                    accumulator: _,
                },
            ) => true,
            (
                OutputFeature::Custom {
                    name: a_name,
                    unit: a_unit,
                    format: _,
                    accumulator: _,
                },
                OutputFeature::Custom {
                    name: b_name,
                    unit: b_unit,
                    format: _,
                    accumulator: _,
                },
            ) => a_name == b_name && a_unit == b_unit,
            _ => false,
        }
    }
}

impl PartialEq<InputFeature> for OutputFeature {
    /// tests equality based on the feature type.
    ///
    /// for distance|time|energy, it's fine to modify either the unit
    /// or the initial value as this should not interfere with properly-
    /// implemented TraversalModel, AccessModel, and FrontierModel instances.
    ///
    /// for custom features, we are stricter about this equality test.
    /// for instance, we cannot allow a user to change the "meaning" of a
    /// state of charge value, that it is a floating point value in the range [0.0, 1.0].
    fn eq(&self, other: &InputFeature) -> bool {
        match (self, other) {
            (
                OutputFeature::Distance {
                    distance_unit: _,
                    initial: _,
                    accumulator: _,
                },
                InputFeature::Distance(_),
            ) => true,
            (
                OutputFeature::Time {
                    time_unit: _,
                    initial: _,
                    accumulator: _,
                },
                InputFeature::Time(_),
            ) => true,
            (
                OutputFeature::Energy {
                    energy_unit: _,
                    initial: _,
                    accumulator: _,
                },
                InputFeature::Energy(_),
            ) => true,
            (
                OutputFeature::Speed {
                    speed_unit: _,
                    initial: _,
                    accumulator: _,
                },
                InputFeature::Speed(_),
            ) => true,
            (
                OutputFeature::Grade {
                    grade_unit: _,
                    initial: _,
                    accumulator: _,
                },
                InputFeature::Grade(_),
            ) => true,
            (
                OutputFeature::Custom {
                    name: a_name,
                    unit: a_unit,
                    format: _,
                    accumulator: _,
                },
                InputFeature::Custom {
                    name: b_name,
                    unit: b_unit,
                },
            ) => a_name == b_name && a_unit == b_unit,
            _ => false,
        }
    }
}

impl Display for OutputFeature {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OutputFeature::Distance {
                distance_unit,
                initial,
                accumulator,
            } => write!(
                f,
                "unit: {}, initial: {}, acc: {}",
                distance_unit, initial, accumulator
            ),
            OutputFeature::Time {
                time_unit,
                initial,
                accumulator,
            } => {
                write!(
                    f,
                    "unit: {}, initial: {}, acc: {}",
                    time_unit, initial, accumulator
                )
            }
            OutputFeature::Energy {
                energy_unit,
                initial,
                accumulator,
            } => write!(
                f,
                "unit: {}, initial: {}, acc: {}",
                energy_unit, initial, accumulator
            ),
            OutputFeature::Speed {
                speed_unit,
                initial,
                accumulator,
            } => write!(
                f,
                "unit: {}, initial: {}, acc: {}",
                speed_unit, initial, accumulator
            ),
            OutputFeature::Grade {
                grade_unit,
                initial,
                accumulator,
            } => write!(
                f,
                "unit: {}, initial: {}, acc: {}",
                grade_unit, initial, accumulator
            ),
            OutputFeature::Custom {
                name,
                unit,
                format,
                accumulator,
            } => {
                write!(
                    f,
                    "name: {} unit: {}, repr: {}, acc: {}",
                    name, unit, format, accumulator
                )
            }
        }
    }
}

impl OutputFeature {
    /// returns true if the feature is an accumulator, aka, that it
    /// can be summed over time and reported in the traversal summary.
    pub fn is_accumlator(&self) -> bool {
        match self {
            OutputFeature::Distance {
                distance_unit: _,
                initial: _,
                accumulator,
            } => *accumulator,
            OutputFeature::Time {
                time_unit: _,
                initial: _,
                accumulator,
            } => *accumulator,
            OutputFeature::Energy {
                energy_unit: _,
                initial: _,
                accumulator,
            } => *accumulator,
            OutputFeature::Speed {
                speed_unit: _,
                initial: _,
                accumulator,
            } => *accumulator,
            OutputFeature::Grade {
                grade_unit: _,
                initial: _,
                accumulator,
            } => *accumulator,
            OutputFeature::Custom {
                name: _,
                unit: _,
                format: _,
                accumulator,
            } => *accumulator,
        }
    }

    pub fn get_feature_type(&self) -> String {
        match self {
            OutputFeature::Distance {
                distance_unit: _,
                initial: _,
                accumulator: _,
            } => String::from("distance"),
            OutputFeature::Time {
                time_unit: _,
                initial: _,
                accumulator: _,
            } => String::from("time"),
            OutputFeature::Energy {
                energy_unit: _,
                initial: _,
                accumulator: _,
            } => String::from("energy"),
            OutputFeature::Speed {
                speed_unit: _,
                initial: _,
                accumulator: _,
            } => String::from("speed"),
            OutputFeature::Grade {
                grade_unit: _,
                initial: _,
                accumulator: _,
            } => String::from("grade"),
            OutputFeature::Custom {
                name,
                unit: _,
                format: _,
                accumulator: _,
            } => name.clone(),
        }
    }

    pub fn get_feature_unit_name(&self) -> String {
        match self {
            OutputFeature::Distance {
                distance_unit,
                initial: _,
                accumulator: _,
            } => distance_unit.to_string(),
            OutputFeature::Time {
                time_unit,
                initial: _,
                accumulator: _,
            } => time_unit.to_string(),
            OutputFeature::Energy {
                energy_unit,
                initial: _,
                accumulator: _,
            } => energy_unit.to_string(),
            OutputFeature::Speed {
                speed_unit,
                initial: _,
                accumulator: _,
            } => speed_unit.to_string(),
            OutputFeature::Grade {
                grade_unit,
                initial: _,
                accumulator: _,
            } => grade_unit.to_string(),
            OutputFeature::Custom {
                name: _,
                unit,
                format: _,
                accumulator: _,
            } => unit.clone(),
        }
    }

    /// custom state variable units may have a custom codec
    /// for domains outside of the real number plane.
    /// this is a helper function to support generic use of the codec,
    /// regardless of unit type.
    pub fn get_feature_format(&self) -> &CustomFeatureFormat {
        match self {
            OutputFeature::Custom {
                name: _,
                unit: _,
                format,
                accumulator: _,
            } => format,
            _ => &CustomFeatureFormat::DEFAULT,
        }
    }

    pub fn get_initial(&self) -> Result<StateVariable, StateModelError> {
        match self {
            OutputFeature::Distance {
                distance_unit: _,
                initial,
                accumulator: _,
            } => Ok((*initial).into()),
            OutputFeature::Time {
                time_unit: _,
                initial,
                accumulator: _,
            } => Ok((*initial).into()),
            OutputFeature::Energy {
                energy_unit: _,
                initial,
                accumulator: _,
            } => Ok((*initial).into()),
            OutputFeature::Speed {
                speed_unit: _,
                initial,
                accumulator: _,
            } => Ok((*initial).into()),
            OutputFeature::Grade {
                grade_unit: _,
                initial,
                accumulator: _,
            } => Ok((*initial).into()),
            OutputFeature::Custom {
                name: _,
                unit: _,
                format,
                accumulator: _,
            } => format.initial(),
        }
    }

    pub fn get_distance_unit(&self) -> Result<&unit::DistanceUnit, StateModelError> {
        match self {
            OutputFeature::Distance {
                distance_unit,
                initial: _,
                accumulator: _,
            } => Ok(distance_unit),
            _ => Err(StateModelError::UnexpectedFeatureUnit(
                String::from("distance"),
                self.get_feature_type(),
            )),
        }
    }

    pub fn get_time_unit(&self) -> Result<&unit::TimeUnit, StateModelError> {
        match self {
            OutputFeature::Time {
                time_unit,
                initial: _,
                accumulator: _,
            } => Ok(time_unit),
            _ => Err(StateModelError::UnexpectedFeatureUnit(
                String::from("time"),
                self.get_feature_type(),
            )),
        }
    }

    pub fn get_energy_unit(&self) -> Result<&unit::EnergyUnit, StateModelError> {
        match self {
            OutputFeature::Energy {
                energy_unit,
                initial: _,
                accumulator: _,
            } => Ok(energy_unit),
            _ => Err(StateModelError::UnexpectedFeatureUnit(
                String::from("energy"),
                self.get_feature_type(),
            )),
        }
    }

    pub fn get_speed_unit(&self) -> Result<&unit::SpeedUnit, StateModelError> {
        match self {
            OutputFeature::Speed {
                speed_unit,
                initial: _,
                accumulator: _,
            } => Ok(speed_unit),
            _ => Err(StateModelError::UnexpectedFeatureUnit(
                String::from("speed"),
                self.get_feature_type(),
            )),
        }
    }

    pub fn get_grade_unit(&self) -> Result<&unit::GradeUnit, StateModelError> {
        match self {
            OutputFeature::Grade {
                grade_unit,
                initial: _,
                accumulator: _,
            } => Ok(grade_unit),
            _ => Err(StateModelError::UnexpectedFeatureUnit(
                String::from("grade"),
                self.get_feature_type(),
            )),
        }
    }

    pub fn get_custom_feature_format(&self) -> Result<&CustomFeatureFormat, StateModelError> {
        match self {
            OutputFeature::Custom {
                name: _,
                unit: _,
                format,
                accumulator: _,
            } => Ok(format),
            _ => Err(StateModelError::UnexpectedFeatureUnit(
                self.get_feature_unit_name(),
                self.get_feature_type(),
            )),
        }
    }
}
