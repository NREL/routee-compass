use std::fmt::Display;

use super::{state_model_error::StateModelError, unit_codec_type::UnitCodecType};
use crate::model::state::StateVariable;
use ordered_float::OrderedFloat;
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
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, PartialOrd)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum CustomFeatureFormat {
    FloatingPoint { initial: OrderedFloat<f64> },
    SignedInteger { initial: i64 },
    UnsignedInteger { initial: u64 },
    Boolean { initial: bool },
}

impl Default for CustomFeatureFormat {
    fn default() -> Self {
        Self::DEFAULT
    }
}

impl Display for CustomFeatureFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let initial = self
            .initial()
            .map(|i| format!("{}", i))
            .unwrap_or_else(|_| String::from("<invalid initial argument>"));
        write!(f, "{}: {}", self.name(), initial)
    }
}

impl CustomFeatureFormat {
    pub const DEFAULT: Self = Self::FloatingPoint {
        initial: OrderedFloat(0.0),
    };

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

    pub fn initial(&self) -> Result<StateVariable, StateModelError> {
        match self {
            CustomFeatureFormat::FloatingPoint { initial } => self.encode_f64(initial),
            CustomFeatureFormat::SignedInteger { initial } => self.encode_i64(initial),
            CustomFeatureFormat::UnsignedInteger { initial } => self.encode_u64(initial),
            CustomFeatureFormat::Boolean { initial } => self.encode_bool(initial),
        }
    }

    pub fn encode_f64(&self, value: &f64) -> Result<StateVariable, StateModelError> {
        match self {
            CustomFeatureFormat::FloatingPoint { initial: _ } => Ok(StateVariable(*value)),
            _ => Err(StateModelError::EncodeError(
                UnitCodecType::FloatingPoint.to_string(),
                self.name(),
            )),
        }
    }

    pub fn encode_i64(&self, value: &i64) -> Result<StateVariable, StateModelError> {
        match self {
            CustomFeatureFormat::SignedInteger { initial: _ } => Ok(StateVariable(*value as f64)),
            _ => Err(StateModelError::EncodeError(
                UnitCodecType::SignedInteger.to_string(),
                self.name(),
            )),
        }
    }

    pub fn encode_u64(&self, value: &u64) -> Result<StateVariable, StateModelError> {
        match self {
            CustomFeatureFormat::UnsignedInteger { initial: _ } => Ok(StateVariable(*value as f64)),
            _ => Err(StateModelError::EncodeError(
                UnitCodecType::UnsignedInteger.to_string(),
                self.name(),
            )),
        }
    }

    pub fn encode_bool(&self, value: &bool) -> Result<StateVariable, StateModelError> {
        match self {
            CustomFeatureFormat::Boolean { initial: _ } => {
                if *value {
                    Ok(StateVariable(1.0))
                } else {
                    Ok(StateVariable(0.0))
                }
            }
            _ => Err(StateModelError::EncodeError(
                UnitCodecType::Boolean.to_string(),
                self.name(),
            )),
        }
    }

    pub fn decode_f64(&self, value: &StateVariable) -> Result<f64, StateModelError> {
        match self {
            CustomFeatureFormat::FloatingPoint { initial: _ } => Ok(value.0),
            _ => Err(StateModelError::DecodeError(
                *value,
                UnitCodecType::FloatingPoint.to_string(),
                self.name(),
            )),
        }
    }
    pub fn decode_i64(&self, value: &StateVariable) -> Result<i64, StateModelError> {
        match self {
            CustomFeatureFormat::SignedInteger { initial: _ } => Ok(value.0 as i64),
            _ => Err(StateModelError::DecodeError(
                *value,
                UnitCodecType::SignedInteger.to_string(),
                self.name(),
            )),
        }
    }
    pub fn decode_u64(&self, value: &StateVariable) -> Result<u64, StateModelError> {
        match self {
            CustomFeatureFormat::UnsignedInteger { initial: _ } => {
                if value < &StateVariable::ZERO {
                    Err(StateModelError::ValueError(
                        *value,
                        UnitCodecType::UnsignedInteger.to_string(),
                    ))
                } else {
                    Ok(value.0 as u64)
                }
            }
            _ => Err(StateModelError::DecodeError(
                *value,
                UnitCodecType::UnsignedInteger.to_string(),
                self.name(),
            )),
        }
    }
    pub fn decode_bool(&self, value: &StateVariable) -> Result<bool, StateModelError> {
        match self {
            CustomFeatureFormat::Boolean { initial: _ } => {
                let is_false = value.0 == 0.0;
                if is_false {
                    Ok(false)
                } else {
                    Ok(true)
                }
            }
            _ => Err(StateModelError::DecodeError(
                *value,
                UnitCodecType::Boolean.to_string(),
                self.name(),
            )),
        }
    }
}
