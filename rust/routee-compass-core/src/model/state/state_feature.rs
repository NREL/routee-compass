use serde::{Deserialize, Serialize};
use std::fmt::Display;
use uom::si::f64::*;

use crate::model::{
    state::{CustomFeatureFormat, StateModelError, StateVariable},
    unit::{DistanceUnit, EnergyUnit, RatioUnit, SpeedUnit, TimeUnit},
};

#[derive(Copy, Clone, PartialEq, PartialOrd, Debug, Deserialize, Serialize)]
pub enum StateFeature {
    Distance {
        value: Length,
        accumulator: bool,
        output_unit: Option<DistanceUnit>,
    },
    Time {
        value: Time,
        accumulator: bool,
        output_unit: Option<TimeUnit>,
    },
    Speed {
        value: Velocity,
        accumulator: bool,
        output_unit: Option<SpeedUnit>,
    },
    Energy {
        value: Energy,
        accumulator: bool,
        output_unit: Option<EnergyUnit>,
    },
    Grade {
        value: Ratio,
        accumulator: bool,
        output_unit: Option<RatioUnit>,
    },
    StateOfCharge {
        value: Ratio,
        accumulator: bool,
        output_unit: Option<RatioUnit>,
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
            StateFeature::StateOfCharge { value, .. } => value.get::<uom::si::ratio::ratio>(),
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
            StateFeature::StateOfCharge { accumulator, .. } => *accumulator,
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

    pub fn get_feature_type(&self) -> String {
        match self {
            StateFeature::Distance { .. } => "distance".to_string(),
            StateFeature::Time { .. } => "time".to_string(),
            StateFeature::Speed { .. } => "speed".to_string(),
            StateFeature::Energy { .. } => "energy".to_string(),
            StateFeature::Grade { .. } => "grade".to_string(),
            StateFeature::StateOfCharge { .. } => "state_of_charge".to_string(),
            StateFeature::Custom { .. } => "custom".to_string(),
        }
    }

    pub fn state_variable_to_f64(&self, state_variable: StateVariable) -> f64 {
        match self {
            StateFeature::Distance { output_unit, .. } => {
                output_unit.map_or(state_variable.into(), |unit| {
                    let uom_value = Length::new::<uom::si::length::meter>(state_variable.into());
                    unit.from_uom(uom_value)
                })
            }
            StateFeature::Time { output_unit, .. } => {
                output_unit.map_or(state_variable.into(), |unit| {
                    let uom_value = Time::new::<uom::si::time::second>(state_variable.into());
                    unit.from_uom(uom_value)
                })
            }
            StateFeature::Speed { output_unit, .. } => {
                output_unit.map_or(state_variable.into(), |unit| {
                    let uom_value =
                        Velocity::new::<uom::si::velocity::meter_per_second>(state_variable.into());
                    unit.from_uom(uom_value)
                })
            }
            StateFeature::Energy { output_unit, .. } => {
                output_unit.map_or(state_variable.into(), |unit| {
                    let uom_value = Energy::new::<uom::si::energy::joule>(state_variable.into());
                    unit.from_uom(uom_value)
                })
            }
            StateFeature::Grade { output_unit, .. } => {
                output_unit.map_or(state_variable.into(), |unit| {
                    let uom_value = Ratio::new::<uom::si::ratio::ratio>(state_variable.into());
                    unit.from_uom(uom_value)
                })
            }
            StateFeature::StateOfCharge { output_unit, .. } => {
                output_unit.map_or(state_variable.into(), |unit| {
                    let uom_value = Ratio::new::<uom::si::ratio::ratio>(state_variable.into());
                    unit.from_uom(uom_value)
                })
            }
            StateFeature::Custom { value, .. } => *value,
        }
    }
}

impl Display for StateFeature {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StateFeature::Distance {
                value,
                accumulator,
                output_unit,
            } => {
                write!(
                    f,
                    "Distance: {:?} (Accumulator: {}, Output Unit: {:?})",
                    value, accumulator, output_unit
                )
            }
            StateFeature::Time {
                value,
                accumulator,
                output_unit,
            } => {
                write!(
                    f,
                    "Time: {:?} (Accumulator: {}, Output Unit: {:?})",
                    value, accumulator, output_unit
                )
            }
            StateFeature::Speed {
                value,
                accumulator,
                output_unit,
            } => {
                write!(
                    f,
                    "Speed: {:?} (Accumulator: {}, Output Unit: {:?})",
                    value, accumulator, output_unit
                )
            }
            StateFeature::Energy {
                value,
                accumulator,
                output_unit,
            } => {
                write!(
                    f,
                    "Energy: {:?} (Accumulator: {}, Output Unit: {:?})",
                    value, accumulator, output_unit
                )
            }
            StateFeature::Grade {
                value,
                accumulator,
                output_unit,
            } => {
                write!(
                    f,
                    "Grade: {:?} (Accumulator: {}, Output Unit: {:?})",
                    value, accumulator, output_unit
                )
            }
            StateFeature::StateOfCharge {
                value,
                accumulator,
                output_unit,
            } => {
                write!(
                    f,
                    "StateOfCharge: {:?} (Accumulator: {}, Output Unit: {:?})",
                    value, accumulator, output_unit
                )
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
