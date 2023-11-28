use std::collections::HashMap;
use std::sync::Arc;

use crate::app::compass::config::compass_configuration_field::CompassConfigurationField;
use crate::app::compass::config::config_json_extension::ConfigJsonExtensions;
use routee_compass_core::model::traversal::traversal_model_builder::TraversalModelBuilder;
use routee_compass_core::model::traversal::traversal_model_error::TraversalModelError;
use routee_compass_core::model::traversal::traversal_model_service::TraversalModelService;
use routee_compass_core::util::unit::{DistanceUnit, GradeUnit, SpeedUnit, TimeUnit};
use routee_compass_powertrain::routee::energy_model_service::EnergyModelService;

use super::energy_model_vehicle_builders::VehicleBuilder;

pub struct EnergyModelBuilder {}

impl TraversalModelBuilder for EnergyModelBuilder {
    fn build(
        &self,
        params: &serde_json::Value,
    ) -> Result<Arc<dyn TraversalModelService>, TraversalModelError> {
        let traversal_key = CompassConfigurationField::Traversal.to_string();

        let speed_table_path = params
            .get_config_path(
                String::from("speed_table_input_file"),
                traversal_key.clone(),
            )
            .map_err(|e| TraversalModelError::BuildError(e.to_string()))?;
        let speed_table_speed_unit = params
            .get_config_serde::<SpeedUnit>(
                String::from("speed_table_speed_unit"),
                traversal_key.clone(),
            )
            .map_err(|e| TraversalModelError::BuildError(e.to_string()))?;

        let grade_table_path = params
            .get_config_path_optional(
                String::from("grade_table_input_file"),
                traversal_key.clone(),
            )
            .map_err(|e| TraversalModelError::BuildError(e.to_string()))?;
        let grade_table_grade_unit = params
            .get_config_serde_optional::<GradeUnit>(
                String::from("graph_grade_unit"),
                traversal_key.clone(),
            )
            .map_err(|e| TraversalModelError::BuildError(e.to_string()))?;

        let vehicle_configs = params
            .get_config_array("vehicles".to_string(), traversal_key.clone())
            .map_err(|e| TraversalModelError::BuildError(e.to_string()))?;

        let mut vehicle_library = HashMap::new();

        for vehicle_config in vehicle_configs {
            let vehicle_type = vehicle_config
                .get_config_string(String::from("type"), traversal_key.clone())
                .map_err(|e| TraversalModelError::BuildError(e.to_string()))?;
            let vehicle_builder = VehicleBuilder::from_string(vehicle_type).map_err(|e| {
                TraversalModelError::BuildError(format!("Error building vehicle builder: {}", e))
            })?;
            let vehicle = vehicle_builder
                .build(&vehicle_config)
                .map_err(|e| TraversalModelError::BuildError(e.to_string()))?;
            vehicle_library.insert(vehicle.name(), vehicle);
        }

        let output_time_unit_option = params
            .get_config_serde_optional::<TimeUnit>(
                String::from("output_time_unit"),
                traversal_key.clone(),
            )
            .map_err(|e| TraversalModelError::BuildError(e.to_string()))?;
        let output_distance_unit_option = params
            .get_config_serde_optional::<DistanceUnit>(
                String::from("output_distance_unit"),
                traversal_key.clone(),
            )
            .map_err(|e| TraversalModelError::BuildError(e.to_string()))?;

        let service = EnergyModelService::new(
            &speed_table_path,
            speed_table_speed_unit,
            &grade_table_path,
            grade_table_grade_unit,
            output_time_unit_option,
            output_distance_unit_option,
            vehicle_library,
        )?;

        Ok(Arc::new(service))
    }
}
