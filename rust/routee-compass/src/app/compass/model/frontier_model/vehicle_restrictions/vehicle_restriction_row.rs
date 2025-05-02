use super::{vehicle_parameter::VehicleParameter, ComparisonOperation};
use routee_compass_core::model::{
    frontier::FrontierModelError,
    network::edge_id::EdgeId,
    unit::{Distance, DistanceUnit, Weight, WeightUnit},
};
use serde::Deserialize;
use std::str::FromStr;

#[derive(Debug, Clone, Deserialize)]
pub struct RestrictionRow {
    pub edge_id: EdgeId,
    pub r#type: String,
    pub value: f64,
    pub operation: ComparisonOperation,
    pub unit: String,
}

impl RestrictionRow {
    pub fn to_parameter(&self) -> Result<VehicleParameter, FrontierModelError> {
        match self.r#type.as_str() {
            "height" => Ok(VehicleParameter::Height {
                value: Distance::from(self.value),
                unit: DistanceUnit::from_str(&self.unit).map_err(|e| {
                    FrontierModelError::BuildError(format!(
                        "Unable to parse height unit {}: {}",
                        self.unit, e
                    ))
                })?,
            }),
            "width" => Ok(VehicleParameter::Width {
                value: Distance::from(self.value),
                unit: DistanceUnit::from_str(&self.unit).map_err(|e| {
                    FrontierModelError::BuildError(format!(
                        "Unable to parse height unit {}: {}",
                        self.unit, e
                    ))
                })?,
            }),
            "total_length" => Ok(VehicleParameter::TotalLength {
                value: Distance::from(self.value),
                unit: DistanceUnit::from_str(&self.unit).map_err(|e| {
                    FrontierModelError::BuildError(format!(
                        "Unable to parse height unit {}: {}",
                        self.unit, e
                    ))
                })?,
            }),
            "trailer_length" => Ok(VehicleParameter::TrailerLength {
                value: Distance::from(self.value),
                unit: DistanceUnit::from_str(&self.unit).map_err(|e| {
                    FrontierModelError::BuildError(format!(
                        "Unable to parse height unit {}: {}",
                        self.unit, e
                    ))
                })?,
            }),
            "total_weight" => Ok(VehicleParameter::TotalWeight {
                value: Weight::from(self.value),
                unit: WeightUnit::from_str(&self.unit).map_err(|e| {
                    FrontierModelError::BuildError(format!(
                        "Unable to parse weight unit {}: {}",
                        self.unit, e
                    ))
                })?,
            }),
            "number_of_axles" => {
                let value = f64_to_u8_safe(self.value)?;
                Ok(VehicleParameter::NumberOfAxles { value })
            }
            _ => Err(FrontierModelError::BuildError(format!(
                "Unknown restriction name {}",
                self.r#type
            ))),
        }
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
