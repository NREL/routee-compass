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
#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum StateFeature {
    Distance { unit: unit::DistanceUnit },
    Time { unit: unit::TimeUnit },
    Liquid { unit: unit::EnergyUnit },
    Electric { unit: unit::EnergyUnit },
    Custom { name: String, codec: UnitCodec },
}

impl StateFeature {
    pub fn get_feature_name(&self) -> String {
        match self {
            StateFeature::Distance { unit: _ } => String::from("distance"),
            StateFeature::Time { unit: _ } => String::from("time"),
            StateFeature::Liquid { unit: _ } => String::from("energy_liquid"),
            StateFeature::Electric { unit: _ } => String::from("energy_electric"),
            StateFeature::Custom { name, codec: _ } => name.clone(),
        }
    }

    /// custom state variable units may have a custom codec
    /// for domains outside of the real number plane.
    /// this is a helper function to support generic use of the codec,
    /// regardless of unit type.
    pub fn get_codec(&self) -> UnitCodec {
        match self {
            StateFeature::Custom { name: _, codec } => *codec,
            _ => UnitCodec::FloatingPoint,
        }
    }

    pub fn get_distance_unit(&self) -> Result<unit::DistanceUnit, StateError> {
        match self {
            StateFeature::Distance { unit } => Ok(*unit),
            _ => Err(StateError::UnexpectedFeatureUnit(
                String::from("distance"),
                self.get_feature_name(),
            )),
        }
    }

    pub fn get_time_unit(&self) -> Result<unit::TimeUnit, StateError> {
        match self {
            StateFeature::Time { unit } => Ok(*unit),
            _ => Err(StateError::UnexpectedFeatureUnit(
                String::from("time"),
                self.get_feature_name(),
            )),
        }
    }

    pub fn get_energy_electric_unit(&self) -> Result<unit::EnergyUnit, StateError> {
        match self {
            StateFeature::Electric { unit } => Ok(*unit),
            _ => Err(StateError::UnexpectedFeatureUnit(
                String::from("energy_electric"),
                self.get_feature_name(),
            )),
        }
    }

    pub fn get_energy_liquid_unit(&self) -> Result<unit::EnergyUnit, StateError> {
        match self {
            StateFeature::Liquid { unit } => Ok(*unit),
            _ => Err(StateError::UnexpectedFeatureUnit(
                String::from("energy_liquid"),
                self.get_feature_name(),
            )),
        }
    }
}
