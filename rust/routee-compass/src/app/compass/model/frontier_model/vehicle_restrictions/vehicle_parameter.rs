use routee_compass_core::model::unit::Convert;
use routee_compass_core::model::unit::{Distance, DistanceUnit, Weight, WeightUnit};
use serde::{Deserialize, Serialize};
use std::borrow::Cow;

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum VehicleParameter {
    Height { value: Distance, unit: DistanceUnit },
    Width { value: Distance, unit: DistanceUnit },
    TotalLength { value: Distance, unit: DistanceUnit },
    TrailerLength { value: Distance, unit: DistanceUnit },
    TotalWeight { value: Weight, unit: WeightUnit },
    NumberOfAxles { value: u8 },
}

impl VehicleParameter {
    pub fn name(&self) -> String {
        match self {
            VehicleParameter::Height { .. } => "height".to_string(),
            VehicleParameter::Width { .. } => "width".to_string(),
            VehicleParameter::TotalLength { .. } => "total_length".to_string(),
            VehicleParameter::TrailerLength { .. } => "trailer_length".to_string(),
            VehicleParameter::TotalWeight { .. } => "total_weight".to_string(),
            VehicleParameter::NumberOfAxles { .. } => "number_of_axles".to_string(),
        }
    }
}

impl std::fmt::Display for VehicleParameter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VehicleParameter::Height { value, unit } => write!(f, "height: {} {}", value, unit),
            VehicleParameter::Width { value, unit } => write!(f, "width: {} {}", value, unit),
            VehicleParameter::TotalLength { value, unit } => {
                write!(f, "total length: {} {}", value, unit)
            }
            VehicleParameter::TrailerLength { value, unit } => {
                write!(f, "trailer length: {} {}", value, unit)
            }
            VehicleParameter::TotalWeight { value, unit } => {
                write!(f, "total weight: {} {}", value, unit)
            }
            VehicleParameter::NumberOfAxles { value } => write!(f, "number of axles: {}", value),
        }
    }
}

impl PartialOrd for VehicleParameter {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match (self, other) {
            (
                VehicleParameter::Height {
                    value: a,
                    unit: a_unit,
                },
                VehicleParameter::Height {
                    value: b,
                    unit: b_unit,
                },
            ) => {
                let mut b_convert = Cow::Borrowed(b);
                b_unit.convert(&mut b_convert, a_unit).ok()?;
                a.partial_cmp(b_convert.as_ref())
            }
            (
                VehicleParameter::Width {
                    value: a,
                    unit: a_unit,
                },
                VehicleParameter::Width {
                    value: b,
                    unit: b_unit,
                },
            ) => {
                let mut b_convert = Cow::Borrowed(b);
                b_unit.convert(&mut b_convert, a_unit).ok()?;
                a.partial_cmp(b_convert.as_ref())
            }
            (
                VehicleParameter::TotalLength {
                    value: a,
                    unit: a_unit,
                },
                VehicleParameter::TotalLength {
                    value: b,
                    unit: b_unit,
                },
            ) => {
                let mut b_convert = Cow::Borrowed(b);
                b_unit.convert(&mut b_convert, a_unit).ok()?;
                a.partial_cmp(b_convert.as_ref())
            }
            (
                VehicleParameter::TrailerLength {
                    value: a,
                    unit: a_unit,
                },
                VehicleParameter::TrailerLength {
                    value: b,
                    unit: b_unit,
                },
            ) => {
                let mut b_convert = Cow::Borrowed(b);
                b_unit.convert(&mut b_convert, a_unit).ok()?;
                a.partial_cmp(b_convert.as_ref())
            }
            (
                VehicleParameter::TotalWeight {
                    value: a,
                    unit: a_unit,
                },
                VehicleParameter::TotalWeight {
                    value: b,
                    unit: b_unit,
                },
            ) => {
                let mut b_convert = Cow::Borrowed(b);
                b_unit.convert(&mut b_convert, a_unit).ok()?;
                a.partial_cmp(b_convert.as_ref())
            }
            (
                VehicleParameter::NumberOfAxles { value: a },
                VehicleParameter::NumberOfAxles { value: b },
            ) => a.partial_cmp(b),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_eq_ordering_matching_parameter() {
        let a = VehicleParameter::Height {
            value: Distance::from(2.0),
            unit: DistanceUnit::Meters,
        };
        let b = VehicleParameter::Height {
            value: Distance::from(2.0),
            unit: DistanceUnit::Meters,
        };
        assert!(a == b);
    }

    #[test]
    fn test_lt_ordering_matching_parameter() {
        let a = VehicleParameter::Height {
            value: Distance::from(2.0),
            unit: DistanceUnit::Meters,
        };
        let b = VehicleParameter::Height {
            value: Distance::from(3.0),
            unit: DistanceUnit::Meters,
        };
        assert!(a < b);
    }

    #[test]
    fn test_lt_ordering_different_parameter() {
        let a = VehicleParameter::TrailerLength {
            value: Distance::from(2.0),
            unit: DistanceUnit::Meters,
        };
        let b = VehicleParameter::Height {
            value: Distance::from(3.0),
            unit: DistanceUnit::Meters,
        };
        // if there is an ordering here, then one of the below must be true
        let has_ordering = a < b || a == b || a > b;
        assert!(!has_ordering);
    }
}
