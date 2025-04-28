use routee_compass_core::config::ConfigJsonExtensions;
use routee_compass_core::model::{
    frontier::FrontierModelError,
    unit::{Distance, DistanceUnit, Weight, WeightUnit},
};

pub struct VehicleParameters {
    pub height: (Distance, DistanceUnit),
    pub width: (Distance, DistanceUnit),
    pub total_length: (Distance, DistanceUnit),
    pub trailer_length: (Distance, DistanceUnit),
    pub total_weight: (Weight, WeightUnit),
    pub number_of_axles: u8,
}

impl VehicleParameters {
    pub fn from_query(query: &serde_json::Value) -> Result<VehicleParameters, FrontierModelError> {
        let vehicle_params = query.get("vehicle_parameters").ok_or_else(|| {
            FrontierModelError::BuildError(
                "Missing field `vehicle_parameters` in query".to_string(),
            )
        })?;

        let height = vehicle_params
            .get_config_serde::<(Distance, DistanceUnit)>(&"height", &"vehicle_parameters")
            .map_err(|e| {
                FrontierModelError::BuildError(format!(
                    "Unable to interpret `height` parameter: {}",
                    e
                ))
            })?;

        let width = vehicle_params
            .get_config_serde::<(Distance, DistanceUnit)>(&"width", &"vehicle_parameters")
            .map_err(|e| {
                FrontierModelError::BuildError(format!(
                    "Unable to interpret `width` parameter: {}",
                    e
                ))
            })?;

        let total_length = vehicle_params
            .get_config_serde::<(Distance, DistanceUnit)>(&"total_length", &"vehicle_parameters")
            .map_err(|e| {
                FrontierModelError::BuildError(format!(
                    "Unable to interpret `total_length` parameter: {}",
                    e
                ))
            })?;

        let trailer_length = vehicle_params
            .get_config_serde::<(Distance, DistanceUnit)>(&"trailer_length", &"vehicle_parameters")
            .map_err(|e| {
                FrontierModelError::BuildError(format!(
                    "Unable to interpret `trailer_length` parameter: {}",
                    e
                ))
            })?;

        let total_weight = vehicle_params
            .get_config_serde::<(Weight, WeightUnit)>(&"total_weight", &"vehicle_parameters")
            .map_err(|e| {
                FrontierModelError::BuildError(format!(
                    "Unable to interpret `total_weight` parameter: {}",
                    e
                ))
            })?;

        let number_of_axles = vehicle_params
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

        let params = VehicleParameters {
            height,
            width,
            total_length,
            trailer_length,
            total_weight,
            number_of_axles,
        };
        Ok(params)
    }
}
