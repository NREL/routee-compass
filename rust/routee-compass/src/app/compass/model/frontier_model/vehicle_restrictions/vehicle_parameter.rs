use routee_compass_core::model::unit::Convert;
use routee_compass_core::model::unit::{Distance, DistanceUnit, Weight, WeightUnit};
use serde::{Deserialize, Serialize};
use std::borrow::Cow;

use crate::app::compass::model::frontier_model::vehicle_restrictions::vehicle_parameter_type::VehicleParameterType;

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum VehicleParameter {
    Height { value: Distance, unit: DistanceUnit },
    Width { value: Distance, unit: DistanceUnit },
    TotalLength { value: Distance, unit: DistanceUnit },
    TrailerLength { value: Distance, unit: DistanceUnit },
    TotalWeight { value: Weight, unit: WeightUnit },
    WeightPerAxle { value: Weight, unit: WeightUnit },
}

impl VehicleParameter {
    pub fn vehicle_parameter_type(&self) -> &VehicleParameterType {
        use VehicleParameterType as VPT;
        match self {
            VehicleParameter::Height { .. } => &VPT::Height,
            VehicleParameter::Width { .. } => &VPT::Width,
            VehicleParameter::TotalLength { .. } => &VPT::TotalLength,
            VehicleParameter::TrailerLength { .. } => &VPT::TrailerLength,
            VehicleParameter::TotalWeight { .. } => &VPT::TotalWeight,
            VehicleParameter::WeightPerAxle { .. } => &VPT::WeightPerAxle,
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
            VehicleParameter::WeightPerAxle { value, unit } => {
                write!(f, "weight per axle: {} {}", value, unit)
            }
        }
    }
}

impl PartialOrd for VehicleParameter {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match (self, other) {
            (
                VehicleParameter::Height { value: a, unit: au },
                VehicleParameter::Height { value: b, unit: bu },
            ) => cmp_params(a, au, b, bu),
            (
                VehicleParameter::Width { value: a, unit: au },
                VehicleParameter::Width { value: b, unit: bu },
            ) => cmp_params(a, au, b, bu),
            (
                VehicleParameter::TotalLength { value: a, unit: au },
                VehicleParameter::TotalLength { value: b, unit: bu },
            ) => cmp_params(a, au, b, bu),
            (
                VehicleParameter::TrailerLength { value: a, unit: au },
                VehicleParameter::TrailerLength { value: b, unit: bu },
            ) => cmp_params(a, au, b, bu),
            (
                VehicleParameter::TotalWeight { value: a, unit: au },
                VehicleParameter::TotalWeight { value: b, unit: bu },
            ) => cmp_params(a, au, b, bu),
            (
                VehicleParameter::WeightPerAxle { value: a, unit: au },
                VehicleParameter::WeightPerAxle { value: b, unit: bu },
            ) => cmp_params(a, au, b, bu),
            _ => {
                // invalid comparison when enum variant of self != variant of other
                None
            }
        }
    }
}

/// compares two matching parameter variants, first ensuring their unit types match, then
/// using the quantity's comparison operator.
fn cmp_params<Q, U>(a: &Q, au: &U, b: &Q, bu: &U) -> Option<std::cmp::Ordering>
where
    Q: Clone + PartialOrd + ?Sized,
    U: Convert<Q>,
{
    let mut b_cmp = Cow::Borrowed(b);
    bu.convert(&mut b_cmp, au).ok()?;
    a.partial_cmp(b_cmp.as_ref())
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
        let has_ordering = a <= b || a > b;
        assert!(!has_ordering);
    }
}
