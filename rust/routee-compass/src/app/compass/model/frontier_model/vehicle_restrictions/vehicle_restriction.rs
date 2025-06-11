use super::{ComparisonOperation, RestrictionRow, VehicleParameter, VehicleParameterType};
use routee_compass_core::model::{
    frontier::FrontierModelError,
    unit::{Distance, DistanceUnit, Weight, WeightUnit},
};
use serde::{Deserialize, Serialize};
use std::{fmt::Display, str::FromStr};

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct VehicleRestriction {
    pub restriction_parameter: VehicleParameter,
    pub comparison_operation: ComparisonOperation,
}

impl VehicleRestriction {
    pub fn new(
        restriction_parameter: VehicleParameter,
        comparison_operation: ComparisonOperation,
    ) -> Self {
        VehicleRestriction {
            restriction_parameter,
            comparison_operation,
        }
    }

    pub fn vehicle_parameter_type(&self) -> &VehicleParameterType {
        self.restriction_parameter.vehicle_parameter_type()
    }

    /// compares this restriction against some query-time vehicle parameter using
    /// the restriction's comparison operator
    pub fn within_restriction(&self, query_parameter: &VehicleParameter) -> bool {
        self.comparison_operation
            .compare_parameters(query_parameter, &self.restriction_parameter)
    }
}

impl TryFrom<&RestrictionRow> for VehicleRestriction {
    type Error = FrontierModelError;

    fn try_from(row: &RestrictionRow) -> Result<Self, Self::Error> {
        use VehicleParameterType as VPT;
        let vehicle_parameter = match row.name {
            VPT::Height => Ok(VehicleParameter::Height {
                value: Distance::from(row.value),
                unit: DistanceUnit::from_str(&row.unit).map_err(|e| {
                    FrontierModelError::BuildError(format!(
                        "Unable to parse height unit {}: {}",
                        row.unit, e
                    ))
                })?,
            }),
            VPT::Width => Ok(VehicleParameter::Width {
                value: Distance::from(row.value),
                unit: DistanceUnit::from_str(&row.unit).map_err(|e| {
                    FrontierModelError::BuildError(format!(
                        "Unable to parse height unit {}: {}",
                        row.unit, e
                    ))
                })?,
            }),
            VPT::TotalLength => Ok(VehicleParameter::TotalLength {
                value: Distance::from(row.value),
                unit: DistanceUnit::from_str(&row.unit).map_err(|e| {
                    FrontierModelError::BuildError(format!(
                        "Unable to parse height unit {}: {}",
                        row.unit, e
                    ))
                })?,
            }),
            VPT::TrailerLength => Ok(VehicleParameter::TrailerLength {
                value: Distance::from(row.value),
                unit: DistanceUnit::from_str(&row.unit).map_err(|e| {
                    FrontierModelError::BuildError(format!(
                        "Unable to parse height unit {}: {}",
                        row.unit, e
                    ))
                })?,
            }),
            VPT::TotalWeight => Ok(VehicleParameter::TotalWeight {
                value: Weight::from(row.value),
                unit: WeightUnit::from_str(&row.unit).map_err(|e| {
                    FrontierModelError::BuildError(format!(
                        "Unable to parse weight unit {}: {}",
                        row.unit, e
                    ))
                })?,
            }),
            VPT::WeightPerAxle => Ok(VehicleParameter::WeightPerAxle {
                value: Weight::from(row.value),
                unit: WeightUnit::from_str(&row.unit).map_err(|e| {
                    FrontierModelError::BuildError(format!(
                        "Unable to parse weight unit {}: {}",
                        row.unit, e
                    ))
                })?,
            }),
        }?;
        let comparison_operation = row.operation.clone();
        Ok(VehicleRestriction {
            restriction_parameter: vehicle_parameter,
            comparison_operation,
        })
    }
}

impl Display for VehicleRestriction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "query parameter is {} link restrictions matching {}",
            self.comparison_operation,
            self.restriction_parameter.vehicle_parameter_type()
        )
    }
}
