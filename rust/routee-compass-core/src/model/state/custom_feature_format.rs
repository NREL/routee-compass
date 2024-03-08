use super::{state_error::StateError, unit_codec_name::UnitCodecType};
use crate::model::traversal::state::state_variable::StateVar;
use serde::{Deserialize, Serialize};

/// codec between StateVar values and basic Rust types.
/// all stateful information in Compass is encoded as a vector of floating
/// point values. a codec provides a mapping to and from types in order
/// to represent different real-numbered and discrete information types.
///
/// because Rust does not support dependent types and trait objects cannot
/// return "Self"s, we are limited in how we generalize over unit types.
/// these codecs substitute for the limitations in the type system, providing
/// a validation check on encoding and decoding to/from StateVar instances
/// and also a codec value that can be instantiated from configuration for a
/// given StateModel.
///
/// warning: this could backfire, but probably in extreme cases that can be avoided.
/// for example, if the user selects esoteric integer values that are not well-represented
/// in floating point, then an encode -> decode roundtrip may not be idempotent.
#[derive(Serialize, Deserialize, Clone, Copy, Debug)]
#[serde(rename_all = "snake_case")]
pub enum CustomFeatureFormat {
    FloatingPoint { initial: f64 },
    SignedInteger { initial: i64 },
    UnsignedInteger { initial: u64 },
    Boolean { initial: bool },
}

impl Default for CustomFeatureFormat {
    fn default() -> Self {
        Self::FloatingPoint { initial: 0.0 }
    }
}

impl CustomFeatureFormat {
    pub fn name(&self) -> String {
        match self {
            CustomFeatureFormat::FloatingPoint { initial: _ } => {
                UnitCodecType::FloatingPoint.to_string()
            }
            CustomFeatureFormat::SignedInteger { initial: _ } => {
                UnitCodecType::SignedInteger.to_string()
            }
            CustomFeatureFormat::UnsignedInteger { initial: _ } => {
                UnitCodecType::UnsignedInteger.to_string()
            }
            CustomFeatureFormat::Boolean { initial: _ } => UnitCodecType::Boolean.to_string(),
        }
    }

    pub fn initial(&self) -> Result<StateVar, StateError> {
        match self {
            CustomFeatureFormat::FloatingPoint { initial } => self.encode_f64(*initial),
            CustomFeatureFormat::SignedInteger { initial } => self.encode_i64(*initial),
            CustomFeatureFormat::UnsignedInteger { initial } => self.encode_u64(*initial),
            CustomFeatureFormat::Boolean { initial } => self.encode_bool(*initial),
        }
    }

    pub fn encode_f64(&self, value: f64) -> Result<StateVar, StateError> {
        match self {
            CustomFeatureFormat::FloatingPoint { initial: _ } => Ok(StateVar(value)),
            _ => Err(StateError::EncodeError(
                UnitCodecType::FloatingPoint.to_string(),
                self.name(),
            )),
        }
    }

    pub fn encode_i64(&self, value: i64) -> Result<StateVar, StateError> {
        match self {
            CustomFeatureFormat::SignedInteger { initial: _ } => Ok(StateVar(value as f64)),
            _ => Err(StateError::EncodeError(
                UnitCodecType::SignedInteger.to_string(),
                self.name(),
            )),
        }
    }

    pub fn encode_u64(&self, value: u64) -> Result<StateVar, StateError> {
        match self {
            CustomFeatureFormat::UnsignedInteger { initial: _ } => Ok(StateVar(value as f64)),
            _ => Err(StateError::EncodeError(
                UnitCodecType::UnsignedInteger.to_string(),
                self.name(),
            )),
        }
    }

    pub fn encode_bool(&self, value: bool) -> Result<StateVar, StateError> {
        match self {
            CustomFeatureFormat::Boolean { initial: _ } => {
                if value {
                    Ok(StateVar(1.0))
                } else {
                    Ok(StateVar(0.0))
                }
            }
            _ => Err(StateError::EncodeError(
                UnitCodecType::Boolean.to_string(),
                self.name(),
            )),
        }
    }

    pub fn as_f64(&self, value: &StateVar) -> Result<f64, StateError> {
        match self {
            CustomFeatureFormat::FloatingPoint { initial: _ } => Ok(value.0),
            _ => Err(StateError::DecodeError(
                *value,
                UnitCodecType::FloatingPoint.to_string(),
                self.name(),
            )),
        }
    }
    pub fn as_i64(&self, value: &StateVar) -> Result<i64, StateError> {
        match self {
            CustomFeatureFormat::SignedInteger { initial: _ } => Ok(value.0 as i64),
            _ => Err(StateError::DecodeError(
                *value,
                UnitCodecType::SignedInteger.to_string(),
                self.name(),
            )),
        }
    }
    pub fn as_u64(&self, value: &StateVar) -> Result<u64, StateError> {
        match self {
            CustomFeatureFormat::UnsignedInteger { initial: _ } => {
                if value < &StateVar::ZERO {
                    Err(StateError::ValueError(
                        *value,
                        UnitCodecType::UnsignedInteger.to_string(),
                    ))
                } else {
                    Ok(value.0 as u64)
                }
            }
            _ => Err(StateError::DecodeError(
                *value,
                UnitCodecType::UnsignedInteger.to_string(),
                self.name(),
            )),
        }
    }
    pub fn as_bool(&self, value: &StateVar) -> Result<bool, StateError> {
        match self {
            CustomFeatureFormat::Boolean { initial: _ } => {
                let is_false = value.0 == 0.0;
                if is_false {
                    Ok(false)
                } else {
                    Ok(true)
                }
            }
            _ => Err(StateError::DecodeError(
                *value,
                UnitCodecType::Boolean.to_string(),
                self.name(),
            )),
        }
    }
}
