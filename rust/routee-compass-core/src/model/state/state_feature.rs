use super::{
    custom_feature_format::CustomFeatureFormat, state_model_error::StateModelError, StateVariable,
};
use crate::model::unit;
use serde::{Deserialize, Serialize};
use std::fmt::Display;

/// a state variable unit tracks the domain of a StateVar in a
/// state vector. if the value represents quantity in distance,
/// time, or energy, then we have a system of internal unit
/// objects which provide conversion arithmetic. if the user
/// specifies a StateVar has a custom state variable unit, then
/// they provide a mapping codec and name for the variable, and
/// it does not interact with our native unit system.
///
/// # Example
///
/// ### Deserialization
///
/// an example TOML representation of state features:
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
pub enum StateFeature {
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
    Custom {
        r#type: String,
        unit: String,
        format: CustomFeatureFormat,
    },
}

impl PartialEq for StateFeature {
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
                StateFeature::Distance {
                    distance_unit: _,
                    initial: _,
                },
                StateFeature::Distance {
                    distance_unit: _,
                    initial: _,
                },
            ) => true,
            (
                StateFeature::Time {
                    time_unit: _,
                    initial: _,
                },
                StateFeature::Time {
                    time_unit: _,
                    initial: _,
                },
            ) => true,
            (
                StateFeature::Energy {
                    energy_unit: _,
                    initial: _,
                },
                StateFeature::Energy {
                    energy_unit: _,
                    initial: _,
                },
            ) => true,
            (
                StateFeature::Custom {
                    r#type: a_name,
                    unit: a_unit,
                    format: _,
                },
                StateFeature::Custom {
                    r#type: b_name,
                    unit: b_unit,
                    format: _,
                },
            ) => a_name == b_name && a_unit == b_unit,
            _ => false,
        }
    }
}

impl Display for StateFeature {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StateFeature::Distance {
                distance_unit,
                initial,
            } => write!(f, "unit: {}, initial: {}", distance_unit, initial),
            StateFeature::Time { time_unit, initial } => {
                write!(f, "unit: {}, initial: {}", time_unit, initial)
            }
            StateFeature::Energy {
                energy_unit,
                initial,
            } => write!(f, "unit: {}, initial: {}", energy_unit, initial),
            StateFeature::Custom {
                r#type: name,
                unit,
                format,
            } => {
                write!(f, "name: {} unit: {}, repr: {}", name, unit, format)
            }
        }
    }
}

impl StateFeature {
    pub fn get_feature_type(&self) -> String {
        match self {
            StateFeature::Distance {
                distance_unit: _,
                initial: _,
            } => String::from("distance"),
            StateFeature::Time {
                time_unit: _,
                initial: _,
            } => String::from("time"),
            StateFeature::Energy {
                energy_unit: _,
                initial: _,
            } => String::from("energy"),
            StateFeature::Custom {
                r#type,
                unit: _,
                format: _,
            } => r#type.clone(),
        }
    }

    pub fn get_feature_unit_name(&self) -> String {
        match self {
            StateFeature::Distance {
                distance_unit,
                initial: _,
            } => distance_unit.to_string(),
            StateFeature::Time {
                time_unit,
                initial: _,
            } => time_unit.to_string(),
            StateFeature::Energy {
                energy_unit,
                initial: _,
            } => energy_unit.to_string(),
            StateFeature::Custom {
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
            StateFeature::Custom {
                r#type: _,
                unit: _,
                format,
            } => format,
            _ => &CustomFeatureFormat::DEFAULT,
        }
    }

    pub fn get_initial(&self) -> Result<StateVariable, StateModelError> {
        match self {
            StateFeature::Distance {
                distance_unit: _,
                initial,
            } => Ok((*initial).into()),
            StateFeature::Time {
                time_unit: _,
                initial,
            } => Ok((*initial).into()),
            StateFeature::Energy {
                energy_unit: _,
                initial,
            } => Ok((*initial).into()),
            StateFeature::Custom {
                r#type: _,
                unit: _,
                format,
            } => format.initial(),
        }
    }

    pub fn get_distance_unit(&self) -> Result<&unit::DistanceUnit, StateModelError> {
        match self {
            StateFeature::Distance {
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
            StateFeature::Time {
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
            StateFeature::Energy {
                energy_unit,
                initial: _,
            } => Ok(energy_unit),
            _ => Err(StateModelError::UnexpectedFeatureUnit(
                String::from("energy"),
                self.get_feature_type(),
            )),
        }
    }

    pub fn get_custom_feature_format(&self) -> Result<&CustomFeatureFormat, StateModelError> {
        match self {
            StateFeature::Custom {
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
