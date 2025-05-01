use std::borrow::Cow;

use routee_compass_core::model::unit::Convert;
use routee_compass_core::model::unit::{Distance, DistanceUnit, Weight, WeightUnit};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "snake_case")]

pub enum VehicleParameter {
    Height { value: Distance, unit: DistanceUnit },
    Width { value: Distance, unit: DistanceUnit },
    TotalLength { value: Distance, unit: DistanceUnit },
    TrailerLength { value: Distance, unit: DistanceUnit },
    TotalWeight { value: Weight, unit: WeightUnit },
    NumberOfAxles(u8),
}

impl VehicleParameter {
    pub fn name(&self) -> String {
        match self {
            VehicleParameter::Height { .. } => "height".to_string(),
            VehicleParameter::Width { .. } => "width".to_string(),
            VehicleParameter::TotalLength { .. } => "total_length".to_string(),
            VehicleParameter::TrailerLength { .. } => "trailer_length".to_string(),
            VehicleParameter::TotalWeight { .. } => "total_weight".to_string(),
            VehicleParameter::NumberOfAxles(_) => "number_of_axles".to_string(),
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
            _ => None,
        }
    }
}
