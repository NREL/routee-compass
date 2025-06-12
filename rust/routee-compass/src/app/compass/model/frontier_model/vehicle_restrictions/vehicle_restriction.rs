use std::str::FromStr;

use super::{ComparisonOperation, RestrictionRow, VehicleParameter};
use routee_compass_core::model::{
    frontier::FrontierModelError,
    unit::{DistanceUnit, WeightUnit},
};
use serde::Deserialize;

#[derive(Debug, Clone, PartialEq, Deserialize)]
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
        let vehicle_parameter = match row.name.as_str() {
            "height" => Ok(VehicleParameter::Height {
                value: DistanceUnit::from_str(&row.unit)
                    .map_err(|e| {
                        FrontierModelError::BuildError(format!(
                            "Unable to parse height unit {}: {}",
                            row.unit, e
                        ))
                    })?
                    .to_uom(row.value),
            }),
            "width" => Ok(VehicleParameter::Width {
                value: DistanceUnit::from_str(&row.unit)
                    .map_err(|e| {
                        FrontierModelError::BuildError(format!(
                            "Unable to parse height unit {}: {}",
                            row.unit, e
                        ))
                    })?
                    .to_uom(row.value),
            }),
            "total_length" => Ok(VehicleParameter::TotalLength {
                value: DistanceUnit::from_str(&row.unit)
                    .map_err(|e| {
                        FrontierModelError::BuildError(format!(
                            "Unable to parse height unit {}: {}",
                            row.unit, e
                        ))
                    })?
                    .to_uom(row.value),
            }),
            "trailer_length" => Ok(VehicleParameter::TrailerLength {
                value: DistanceUnit::from_str(&row.unit)
                    .map_err(|e| {
                        FrontierModelError::BuildError(format!(
                            "Unable to parse height unit {}: {}",
                            row.unit, e
                        ))
                    })?
                    .to_uom(row.value),
            }),
            "total_weight" => Ok(VehicleParameter::TotalWeight {
                value: WeightUnit::from_str(&row.unit)
                    .map_err(|e| {
                        FrontierModelError::BuildError(format!(
                            "Unable to parse weight unit {}: {}",
                            row.unit, e
                        ))
                    })?
                    .to_uom(row.value),
            }),
            "weight_per_axle" => Ok(VehicleParameter::WeightPerAxle {
                value: WeightUnit::from_str(&row.unit)
                    .map_err(|e| {
                        FrontierModelError::BuildError(format!(
                            "Unable to parse weight unit {}: {}",
                            row.unit, e
                        ))
                    })?
                    .to_uom(row.value),
            }),
            _ => Err(FrontierModelError::BuildError(format!(
                "Unknown vehicle parameter type: {}",
                row.name
            ))),
        }?;
        let comparison_operation = row.operation.clone();
        Ok(VehicleRestriction {
            restriction_parameter: vehicle_parameter,
            comparison_operation,
        })
    }
}
