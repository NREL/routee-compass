use serde::{Deserialize, Serialize};
use std::fmt::Display;
use uom::si::f64::*;

use crate::model::state::{CustomFeatureFormat, StateModelError};

#[derive(Copy, Clone, PartialEq, PartialOrd, Debug, Deserialize, Serialize)]
pub enum StateFeature {
    Distance {
        value: Length,
        accumulator: bool,
    },
    Time {
        value: Time,
        accumulator: bool,
    },
    Speed {
        value: Velocity,
        accumulator: bool,
    },
    Energy {
        value: Energy,
        accumulator: bool,
    },
    Grade {
        value: Ratio,
        accumulator: bool,
    },
    Custom {
        value: f64,
        accumulator: bool,
        format: CustomFeatureFormat,
    },
}

impl StateFeature {
    pub fn as_f64(&self) -> f64 {
        match self {
            StateFeature::Distance { value, .. } => value.get::<uom::si::length::meter>(),
            StateFeature::Time { value, .. } => value.get::<uom::si::time::second>(),
            StateFeature::Speed { value, .. } => value.get::<uom::si::velocity::meter_per_second>(),
            StateFeature::Energy { value, .. } => value.get::<uom::si::energy::joule>(),
            StateFeature::Grade { value, .. } => value.get::<uom::si::ratio::ratio>(),
            StateFeature::Custom { value, .. } => *value,
        }
    }
    pub fn is_accumulator(&self) -> bool {
        match self {
            StateFeature::Distance { accumulator, .. } => *accumulator,
            StateFeature::Time { accumulator, .. } => *accumulator,
            StateFeature::Speed { accumulator, .. } => *accumulator,
            StateFeature::Energy { accumulator, .. } => *accumulator,
            StateFeature::Grade { accumulator, .. } => *accumulator,
            StateFeature::Custom { accumulator, .. } => *accumulator,
        }
    }
    pub fn get_custom_feature_format(&self) -> Result<&CustomFeatureFormat, StateModelError> {
        match self {
            StateFeature::Custom { format, .. } => Ok(format),
            _ => Err(StateModelError::UnexpectedFeatureType(
                "Expected Custom feature type".to_string(),
                format!("Got: {:?}", self),
            )),
        }
    }
}

impl Display for StateFeature {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StateFeature::Distance { value, accumulator } => {
                write!(f, "Distance: {:?} (Accumulator: {})", value, accumulator)
            }
            StateFeature::Time { value, accumulator } => {
                write!(f, "Time: {:?} (Accumulator: {})", value, accumulator)
            }
            StateFeature::Speed { value, accumulator } => {
                write!(f, "Speed: {:?} (Accumulator: {})", value, accumulator)
            }
            StateFeature::Energy { value, accumulator } => {
                write!(f, "Energy: {:?} (Accumulator: {})", value, accumulator)
            }
            StateFeature::Grade { value, accumulator } => {
                write!(f, "Grade: {:?} (Accumulator: {})", value, accumulator)
            }
            StateFeature::Custom {
                value,
                accumulator,
                format,
            } => {
                write!(
                    f,
                    "CustomF64: {} (Accumulator: {}, Format: {})",
                    value, accumulator, format
                )
            }
        }
    }
}

