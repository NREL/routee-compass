use super::vehicle_parameters::VehicleParameter;
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
    pub restriction_name: String,
    pub restriction_value: f64,
    pub restriction_unit: String,
}

impl RestrictionRow {
    pub fn to_parameter(&self) -> Result<VehicleParameter, FrontierModelError> {
        match self.restriction_name.as_str() {
            "height" => Ok(VehicleParameter::Height {
                value: Distance::from(self.restriction_value),
                unit: DistanceUnit::from_str(&self.restriction_unit).map_err(|e| {
                    FrontierModelError::BuildError(format!(
                        "Unable to parse height unit {}: {}",
                        self.restriction_unit, e
                    ))
                })?,
            }),
            "width" => Ok(VehicleParameter::Width {
                value: Distance::from(self.restriction_value),
                unit: DistanceUnit::from_str(&self.restriction_unit).map_err(|e| {
                    FrontierModelError::BuildError(format!(
                        "Unable to parse height unit {}: {}",
                        self.restriction_unit, e
                    ))
                })?,
            }),
            "total_length" => Ok(VehicleParameter::TotalLength {
                value: Distance::from(self.restriction_value),
                unit: DistanceUnit::from_str(&self.restriction_unit).map_err(|e| {
                    FrontierModelError::BuildError(format!(
                        "Unable to parse height unit {}: {}",
                        self.restriction_unit, e
                    ))
                })?,
            }),
            "trailer_length" => Ok(VehicleParameter::TrailerLength {
                value: Distance::from(self.restriction_value),
                unit: DistanceUnit::from_str(&self.restriction_unit).map_err(|e| {
                    FrontierModelError::BuildError(format!(
                        "Unable to parse height unit {}: {}",
                        self.restriction_unit, e
                    ))
                })?,
            }),
            "total_weight" => Ok(VehicleParameter::TotalWeight {
                value: Weight::from(self.restriction_value),
                unit: WeightUnit::from_str(&self.restriction_unit).map_err(|e| {
                    FrontierModelError::BuildError(format!(
                        "Unable to parse weight unit {}: {}",
                        self.restriction_unit, e
                    ))
                })?,
            }),
            "number_of_axles" => Ok(VehicleParameter::NumberOfAxles(
                self.restriction_value as u8,
            )),
            _ => Err(FrontierModelError::BuildError(format!(
                "Unknown restriction name {}",
                self.restriction_name
            ))),
        }
    }
}
