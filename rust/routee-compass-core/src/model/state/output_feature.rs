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
    },
    Time {
        time_unit: unit::TimeUnit,
        initial: unit::Time,
    },
    Energy {
        energy_unit: unit::EnergyUnit,
        initial: unit::Energy,
    },
    Speed {
        speed_unit: unit::SpeedUnit,
        initial: unit::Speed,
    },
    Grade {
        grade_unit: unit::GradeUnit,
        initial: unit::Grade,
    },
    Custom {
        r#type: String,
        unit: String,
        format: CustomFeatureFormat,
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
                },
                OutputFeature::Distance {
                    distance_unit: _,
                    initial: _,
                },
            ) => true,
            (
                OutputFeature::Time {
                    time_unit: _,
                    initial: _,
                },
                OutputFeature::Time {
                    time_unit: _,
                    initial: _,
                },
            ) => true,
            (
                OutputFeature::Energy {
                    energy_unit: _,
                    initial: _,
                },
                OutputFeature::Energy {
                    energy_unit: _,
                    initial: _,
                },
            ) => true,
            (
                OutputFeature::Speed {
                    speed_unit: _,
                    initial: _,
                },
                OutputFeature::Speed {
                    speed_unit: _,
                    initial: _,
                },
            ) => true,
            (
                OutputFeature::Grade {
                    grade_unit: _,
                    initial: _,
                },
                OutputFeature::Grade {
                    grade_unit: _,
                    initial: _,
                },
            ) => true,
            (
                OutputFeature::Custom {
                    r#type: a_name,
                    unit: a_unit,
                    format: _,
                },
                OutputFeature::Custom {
                    r#type: b_name,
                    unit: b_unit,
                    format: _,
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
                },
                InputFeature::Distance(_),
            ) => true,
            (
                OutputFeature::Time {
                    time_unit: _,
                    initial: _,
                },
                InputFeature::Time(_),
            ) => true,
            (
                OutputFeature::Energy {
                    energy_unit: _,
                    initial: _,
                },
                InputFeature::Energy(_),
            ) => true,
            (
                OutputFeature::Speed {
                    speed_unit: _,
                    initial: _,
                },
                InputFeature::Speed(_),
            ) => true,
            (
                OutputFeature::Grade {
                    grade_unit: _,
                    initial: _,
                },
                InputFeature::Grade(_),
            ) => true,
            (
                OutputFeature::Custom {
                    r#type: a_name,
                    unit: a_unit,
                    format: _,
                },
                InputFeature::Custom {
                    r#type: b_name,
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
            } => write!(f, "unit: {}, initial: {}", distance_unit, initial),
            OutputFeature::Time { time_unit, initial } => {
                write!(f, "unit: {}, initial: {}", time_unit, initial)
            }
            OutputFeature::Energy {
                energy_unit,
                initial,
            } => write!(f, "unit: {}, initial: {}", energy_unit, initial),
            OutputFeature::Speed {
                speed_unit,
                initial,
            } => write!(f, "unit: {}, initial: {}", speed_unit, initial),
            OutputFeature::Grade {
                grade_unit,
                initial,
            } => write!(f, "unit: {}, initial: {}", grade_unit, initial),
            OutputFeature::Custom {
                r#type: name,
                unit,
                format,
            } => {
                write!(f, "name: {} unit: {}, repr: {}", name, unit, format)
            }
        }
    }
}

impl OutputFeature {
    pub fn get_feature_type(&self) -> String {
        match self {
            OutputFeature::Distance {
                distance_unit: _,
                initial: _,
            } => String::from("distance"),
            OutputFeature::Time {
                time_unit: _,
                initial: _,
            } => String::from("time"),
            OutputFeature::Energy {
                energy_unit: _,
                initial: _,
            } => String::from("energy"),
            OutputFeature::Speed {
                speed_unit: _,
                initial: _,
            } => String::from("speed"),
            OutputFeature::Grade {
                grade_unit: _,
                initial: _,
            } => String::from("grade"),
            OutputFeature::Custom {
                r#type,
                unit: _,
                format: _,
            } => r#type.clone(),
        }
    }

    pub fn get_feature_unit_name(&self) -> String {
        match self {
            OutputFeature::Distance {
                distance_unit,
                initial: _,
            } => distance_unit.to_string(),
            OutputFeature::Time {
                time_unit,
                initial: _,
            } => time_unit.to_string(),
            OutputFeature::Energy {
                energy_unit,
                initial: _,
            } => energy_unit.to_string(),
            OutputFeature::Speed {
                speed_unit,
                initial: _,
            } => speed_unit.to_string(),
            OutputFeature::Grade {
                grade_unit,
                initial: _,
            } => grade_unit.to_string(),
            OutputFeature::Custom {
                r#type: _,
                unit,
                format: _,
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
                r#type: _,
                unit: _,
                format,
            } => format,
            _ => &CustomFeatureFormat::DEFAULT,
        }
    }

    pub fn get_initial(&self) -> Result<StateVariable, StateModelError> {
        match self {
            OutputFeature::Distance {
                distance_unit: _,
                initial,
            } => Ok((*initial).into()),
            OutputFeature::Time {
                time_unit: _,
                initial,
            } => Ok((*initial).into()),
            OutputFeature::Energy {
                energy_unit: _,
                initial,
            } => Ok((*initial).into()),
            OutputFeature::Speed {
                speed_unit: _,
                initial,
            } => Ok((*initial).into()),
            OutputFeature::Grade {
                grade_unit: _,
                initial,
            } => Ok((*initial).into()),
            OutputFeature::Custom {
                r#type: _,
                unit: _,
                format,
            } => format.initial(),
        }
    }

    pub fn get_distance_unit(&self) -> Result<&unit::DistanceUnit, StateModelError> {
        match self {
            OutputFeature::Distance {
                distance_unit,
                initial: _,
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
                r#type: _,
                unit: _,
                format,
            } => Ok(format),
            _ => Err(StateModelError::UnexpectedFeatureUnit(
                self.get_feature_unit_name(),
                self.get_feature_type(),
            )),
        }
    }
}
