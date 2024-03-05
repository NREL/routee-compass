use super::state_error::StateError;
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
#[derive(Serialize, Deserialize, Clone, Copy)]
#[serde(rename_all = "snake_case")]
pub enum UnitCodec {
    FloatingPoint,
    SignedInteger,
    UnsignedInteger,
    Boolean,
}

impl UnitCodec {
    pub fn name(&self) -> String {
        match self {
            UnitCodec::FloatingPoint => String::from("floating_point"),
            UnitCodec::SignedInteger => String::from("signed_integer"),
            UnitCodec::UnsignedInteger => String::from("unsigned_integer"),
            UnitCodec::Boolean => String::from("boolean"),
        }
    }

    pub fn encode_f64(&self, value: f64) -> Result<StateVar, StateError> {
        match self {
            UnitCodec::FloatingPoint => Ok(StateVar(value)),
            _ => Err(StateError::EncodeError(
                UnitCodec::FloatingPoint.name(),
                self.name(),
            )),
        }
    }

    pub fn encode_i64(&self, value: i64) -> Result<StateVar, StateError> {
        match self {
            UnitCodec::SignedInteger => Ok(StateVar(value as f64)),
            _ => Err(StateError::EncodeError(
                UnitCodec::SignedInteger.name(),
                self.name(),
            )),
        }
    }

    pub fn encode_u64(&self, value: u64) -> Result<StateVar, StateError> {
        match self {
            UnitCodec::UnsignedInteger => Ok(StateVar(value as f64)),
            _ => Err(StateError::EncodeError(
                UnitCodec::UnsignedInteger.name(),
                self.name(),
            )),
        }
    }

    pub fn encode_bool(&self, value: bool) -> Result<StateVar, StateError> {
        match self {
            UnitCodec::Boolean => {
                if value {
                    Ok(StateVar(1.0))
                } else {
                    Ok(StateVar(0.0))
                }
            }
            _ => Err(StateError::EncodeError(
                UnitCodec::Boolean.name(),
                self.name(),
            )),
        }
    }

    pub fn as_f64(&self, value: &StateVar) -> Result<f64, StateError> {
        match self {
            UnitCodec::FloatingPoint => Ok(value.0),
            _ => Err(StateError::DecodeError(
                *value,
                UnitCodec::FloatingPoint.name(),
                self.name(),
            )),
        }
    }
    pub fn as_i64(&self, value: &StateVar) -> Result<i64, StateError> {
        match self {
            UnitCodec::SignedInteger => Ok(value.0 as i64),
            _ => Err(StateError::DecodeError(
                *value,
                UnitCodec::SignedInteger.name(),
                self.name(),
            )),
        }
    }
    pub fn as_u64(&self, value: &StateVar) -> Result<u64, StateError> {
        match self {
            UnitCodec::UnsignedInteger => {
                if value < &StateVar::ZERO {
                    Err(StateError::ValueError(
                        *value,
                        UnitCodec::UnsignedInteger.name(),
                    ))
                } else {
                    Ok(value.0 as u64)
                }
            }
            _ => Err(StateError::DecodeError(
                *value,
                UnitCodec::UnsignedInteger.name(),
                self.name(),
            )),
        }
    }
    pub fn as_bool(&self, value: &StateVar) -> Result<bool, StateError> {
        match self {
            UnitCodec::Boolean => {
                let is_false = value.0 == 0.0;
                if is_false {
                    Ok(false)
                } else {
                    Ok(true)
                }
            }
            _ => Err(StateError::DecodeError(
                *value,
                UnitCodec::Boolean.name(),
                self.name(),
            )),
        }
    }
}
