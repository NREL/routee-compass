use super::unit_codec::UnitCodec;
use crate::model::unit;
use serde::{Deserialize, Serialize};

/// a state variable unit tracks the domain of a StateVar in a
/// state vector. if the value represents quantity in distance,
/// time, or energy, then we have a system of internal unit
/// objects which provide conversion arithmetic. if the user
/// specifies a StateVar has a custom state variable unit, then
/// they provide a mapping codec and name for the variable, and
/// it does not interact with our native unit system.
#[derive(Serialize, Deserialize, Clone)]
#[serde(tag = "type")]
pub enum StateVariableUnit {
    Distance { unit: unit::DistanceUnit },
    Time { unit: unit::TimeUnit },
    Liquid { unit: unit::EnergyUnit },
    Electric { unit: unit::EnergyUnit },
    Custom { name: String, codec: UnitCodec },
}

impl StateVariableUnit {
    /// custom state variable units may have a custom codec
    /// for domains outside of the real number plane.
    /// this is a helper function to support generic use of the codec,
    /// regardless of unit type.
    pub fn get_codec(&self) -> UnitCodec {
        match self {
            StateVariableUnit::Custom { name: _, codec } => *codec,
            _ => UnitCodec::FloatingPoint,
        }
    }
}
