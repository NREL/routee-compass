use super::{custom_feature_format::CustomFeatureFormat, state_error::StateError};
use crate::model::{traversal::state::state_variable::StateVar, unit};
use serde::{Deserialize, Serialize};

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
#[derive(Serialize, Deserialize, Clone)]
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
    Liquid {
        energy_liquid_unit: unit::EnergyUnit,
        initial: unit::Energy,
    },
    Electric {
        energy_electric_unit: unit::EnergyUnit,
        initial: unit::Energy,
    },
    Custom {
        name: String,
        unit: String,
        format: CustomFeatureFormat,
    },
}

impl StateFeature {
    pub fn get_feature_name(&self) -> String {
        match self {
            StateFeature::Distance {
                distance_unit: _,
                initial: _,
            } => String::from("distance"),
            StateFeature::Time {
                time_unit: _,
                initial: _,
            } => String::from("time"),
            StateFeature::Liquid {
                energy_liquid_unit: _,
                initial: _,
            } => String::from("energy_liquid"),
            StateFeature::Electric {
                energy_electric_unit: _,
                initial: _,
            } => String::from("energy_electric"),
            StateFeature::Custom {
                name,
                unit: _,
                format: _,
            } => name.clone(),
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
            StateFeature::Liquid {
                energy_liquid_unit,
                initial: _,
            } => energy_liquid_unit.to_string(),
            StateFeature::Electric {
                energy_electric_unit,
                initial: _,
            } => energy_electric_unit.to_string(),
            StateFeature::Custom {
                name: _,
                unit,
                format: _,
            } => unit.clone(),
        }
    }

    /// custom state variable units may have a custom codec
    /// for domains outside of the real number plane.
    /// this is a helper function to support generic use of the codec,
    /// regardless of unit type.
    pub fn get_feature_format(&self) -> CustomFeatureFormat {
        match self {
            StateFeature::Custom {
                name: _,
                unit: _,
                format,
            } => *format,
            _ => CustomFeatureFormat::default(),
        }
    }

    pub fn get_initial(&self) -> Result<StateVar, StateError> {
        self.get_feature_format().initial()
    }

    pub fn get_distance_unit(&self) -> Result<unit::DistanceUnit, StateError> {
        match self {
            StateFeature::Distance {
                distance_unit: unit,
                initial: _,
            } => Ok(*unit),
            _ => Err(StateError::UnexpectedFeatureUnit(
                String::from("distance"),
                self.get_feature_name(),
            )),
        }
    }

    pub fn get_time_unit(&self) -> Result<unit::TimeUnit, StateError> {
        match self {
            StateFeature::Time {
                time_unit: unit,
                initial: _,
            } => Ok(*unit),
            _ => Err(StateError::UnexpectedFeatureUnit(
                String::from("time"),
                self.get_feature_name(),
            )),
        }
    }

    pub fn get_energy_electric_unit(&self) -> Result<unit::EnergyUnit, StateError> {
        match self {
            StateFeature::Electric {
                energy_electric_unit: unit,
                initial: _,
            } => Ok(*unit),
            _ => Err(StateError::UnexpectedFeatureUnit(
                String::from("energy_electric"),
                self.get_feature_name(),
            )),
        }
    }

    pub fn get_energy_liquid_unit(&self) -> Result<unit::EnergyUnit, StateError> {
        match self {
            StateFeature::Liquid {
                energy_liquid_unit: unit,
                initial: _,
            } => Ok(*unit),
            _ => Err(StateError::UnexpectedFeatureUnit(
                String::from("energy_liquid"),
                self.get_feature_name(),
            )),
        }
    }
}
