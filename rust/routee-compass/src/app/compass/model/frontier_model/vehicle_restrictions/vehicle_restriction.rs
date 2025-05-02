use std::str::FromStr;

use super::{ComparisonOperation, RestrictionRow, VehicleParameter};
use routee_compass_core::model::{
    frontier::FrontierModelError,
    unit::{Distance, DistanceUnit, Weight, WeightUnit},
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct VehicleRestriction {
    pub restriction_parameter: VehicleParameter,
    pub comparison_operation: ComparisonOperation,
}

impl VehicleRestriction {
    pub fn new(
        vehicle_parameter: VehicleParameter,
        comparison_operation: ComparisonOperation,
    ) -> Self {
        VehicleRestriction {
            restriction_parameter: vehicle_parameter,
            comparison_operation,
        }
    }

    pub fn name(&self) -> String {
        self.restriction_parameter.name()
    }

    pub fn validate_parameters(&self, query_parameter: &VehicleParameter) -> bool {
        self.comparison_operation
            .compare_parameters(query_parameter, &self.restriction_parameter)
    }
}

impl TryFrom<&RestrictionRow> for VehicleRestriction {
    type Error = FrontierModelError;

    fn try_from(row: &RestrictionRow) -> Result<Self, Self::Error> {
        let vehicle_parameter = match row.r#type.as_str() {
            "height" => Ok(VehicleParameter::Height {
                value: Distance::from(row.value),
                unit: DistanceUnit::from_str(&row.unit).map_err(|e| {
                    FrontierModelError::BuildError(format!(
                        "Unable to parse height unit {}: {}",
                        row.unit, e
                    ))
                })?,
            }),
            "width" => Ok(VehicleParameter::Width {
                value: Distance::from(row.value),
                unit: DistanceUnit::from_str(&row.unit).map_err(|e| {
                    FrontierModelError::BuildError(format!(
                        "Unable to parse height unit {}: {}",
                        row.unit, e
                    ))
                })?,
            }),
            "total_length" => Ok(VehicleParameter::TotalLength {
                value: Distance::from(row.value),
                unit: DistanceUnit::from_str(&row.unit).map_err(|e| {
                    FrontierModelError::BuildError(format!(
                        "Unable to parse height unit {}: {}",
                        row.unit, e
                    ))
                })?,
            }),
            "trailer_length" => Ok(VehicleParameter::TrailerLength {
                value: Distance::from(row.value),
                unit: DistanceUnit::from_str(&row.unit).map_err(|e| {
                    FrontierModelError::BuildError(format!(
                        "Unable to parse height unit {}: {}",
                        row.unit, e
                    ))
                })?,
            }),
            "total_weight" => Ok(VehicleParameter::TotalWeight {
                value: Weight::from(row.value),
                unit: WeightUnit::from_str(&row.unit).map_err(|e| {
                    FrontierModelError::BuildError(format!(
                        "Unable to parse weight unit {}: {}",
                        row.unit, e
                    ))
                })?,
            }),
            "number_of_axles" => {
                let value = f64_to_u8_safe(row.value)?;
                Ok(VehicleParameter::NumberOfAxles { value })
            }
            _ => Err(FrontierModelError::BuildError(format!(
                "Unknown restriction name {}",
                row.r#type
            ))),
        }?;
        let comparison_operation = row.operation.clone();
        Ok(VehicleRestriction {
            restriction_parameter: vehicle_parameter,
            comparison_operation,
        })
    }
}

fn f64_to_u8_safe(value: f64) -> Result<u8, FrontierModelError> {
    if value >= 0.0 && value <= u8::MAX as f64 {
        Ok(value as u8)
    } else {
        Err(FrontierModelError::BuildError(format!(
            "Value {} is out of range for u8",
            value
        )))
    }
}
