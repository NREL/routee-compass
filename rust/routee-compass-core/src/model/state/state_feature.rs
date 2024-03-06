use super::{state_error::StateError, unit_codec::UnitCodec};
use crate::model::unit;
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
///   { "distance_unit" = "kilometers" },
///   { "time_unit" = "minutes" },
///   { "custom_feature_name" = "soc", codec = "floating_point" }
/// ]
/// ```
#[derive(Serialize, Deserialize)]
pub enum StateFeature {
    Distance {
        distance_unit: unit::DistanceUnit,
    },
    Time {
        time_unit: unit::TimeUnit,
    },
    Liquid {
        energy_liquid_unit: unit::EnergyUnit,
    },
    Electric {
        energy_electric_unit: unit::EnergyUnit,
    },
    Custom {
        custom_feature_name: String,
        codec: UnitCodec,
    },
}

impl StateFeature {
    pub fn get_feature_name(&self) -> String {
        match self {
            StateFeature::Distance { distance_unit: _ } => String::from("distance"),
            StateFeature::Time { time_unit: _ } => String::from("time"),
            StateFeature::Liquid {
                energy_liquid_unit: _,
            } => String::from("energy_liquid"),
            StateFeature::Electric {
                energy_electric_unit: _,
            } => String::from("energy_electric"),
            StateFeature::Custom {
                custom_feature_name: name,
                codec: _,
            } => name.clone(),
        }
    }

    /// custom state variable units may have a custom codec
    /// for domains outside of the real number plane.
    /// this is a helper function to support generic use of the codec,
    /// regardless of unit type.
    pub fn get_codec(&self) -> UnitCodec {
        match self {
            StateFeature::Custom {
                custom_feature_name: _,
                codec,
            } => *codec,
            _ => UnitCodec::FloatingPoint,
        }
    }

    pub fn get_distance_unit(&self) -> Result<unit::DistanceUnit, StateError> {
        match self {
            StateFeature::Distance {
                distance_unit: unit,
            } => Ok(*unit),
            _ => Err(StateError::UnexpectedFeatureUnit(
                String::from("distance"),
                self.get_feature_name(),
            )),
        }
    }

    pub fn get_time_unit(&self) -> Result<unit::TimeUnit, StateError> {
        match self {
            StateFeature::Time { time_unit: unit } => Ok(*unit),
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
            } => Ok(*unit),
            _ => Err(StateError::UnexpectedFeatureUnit(
                String::from("energy_liquid"),
                self.get_feature_name(),
            )),
        }
    }
}
