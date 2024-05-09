use crate::app::compass::config::config_json_extension::ConfigJsonExtensions;

use routee_compass_core::model::{
    frontier::frontier_model_error::FrontierModelError,
    unit::{Distance, DistanceUnit, Weight, WeightUnit},
};

pub struct TruckParameters {
    pub truck_height: (Distance, DistanceUnit),
    pub truck_width: (Distance, DistanceUnit),
    pub truck_total_length: (Distance, DistanceUnit),
    pub truck_trailer_length: (Distance, DistanceUnit),
    pub truck_total_weight: (Weight, WeightUnit),
    pub truck_number_of_axles: u8,
}

impl TruckParameters {
    pub fn from_query(query: &serde_json::Value) -> Result<TruckParameters, FrontierModelError> {
        let truck_params = query.get("truck_parameters").ok_or_else(|| {
            FrontierModelError::BuildError("Missing field `truck_parameters` in query".to_string())
        })?;

        let height = truck_params
            .get_config_serde::<(Distance, DistanceUnit)>(&"height", &"truck_parameters")
            .map_err(|e| {
                FrontierModelError::BuildError(format!(
                    "Unable to interpret `height` parameter: {}",
                    e
                ))
            })?;

        let width = truck_params
            .get_config_serde::<(Distance, DistanceUnit)>(&"width", &"truck_parameters")
            .map_err(|e| {
                FrontierModelError::BuildError(format!(
                    "Unable to interpret `width` parameter: {}",
                    e
                ))
            })?;

        let total_length = truck_params
            .get_config_serde::<(Distance, DistanceUnit)>(&"total_length", &"truck_parameters")
            .map_err(|e| {
                FrontierModelError::BuildError(format!(
                    "Unable to interpret `total_length` parameter: {}",
                    e
                ))
            })?;

        let trailer_length = truck_params
            .get_config_serde::<(Distance, DistanceUnit)>(&"trailer_length", &"truck_parameters")
            .map_err(|e| {
                FrontierModelError::BuildError(format!(
                    "Unable to interpret `trailer_length` parameter: {}",
                    e
                ))
            })?;

        let total_weight = truck_params
            .get_config_serde::<(Weight, WeightUnit)>(&"total_weight", &"truck_parameters")
            .map_err(|e| {
                FrontierModelError::BuildError(format!(
                    "Unable to interpret `total_weight` parameter: {}",
                    e
                ))
            })?;

        let number_of_axles = truck_params
            .get("number_of_axles")
            .ok_or_else(|| {
                FrontierModelError::BuildError(
                    "Missing field `number_of_axles` in query".to_string(),
                )
            })?
            .as_u64()
            .ok_or_else(|| {
                FrontierModelError::BuildError(
                    "Unable to interpret `number_of_axles` parameter as an integer".to_string(),
                )
            })? as u8;

        let params = TruckParameters {
            truck_height: height,
            truck_width: width,
            truck_total_length: total_length,
            truck_trailer_length: trailer_length,
            truck_total_weight: total_weight,
            truck_number_of_axles: number_of_axles,
        };
        Ok(params)
    }
}
