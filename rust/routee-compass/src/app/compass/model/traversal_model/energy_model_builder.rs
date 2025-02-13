use itertools::Itertools;
use routee_compass_core::config::ConfigJsonExtensions;
use routee_compass_core::model::traversal::TraversalModelBuilder;
use routee_compass_core::model::traversal::TraversalModelError;
use routee_compass_core::model::traversal::TraversalModelService;
use routee_compass_core::model::unit::{DistanceUnit, GradeUnit, SpeedUnit, TimeUnit};
use routee_compass_powertrain::model::energy_model_service::EnergyModelService;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::Arc;

use super::energy_model_vehicle_builders::VehicleBuilder;

pub struct EnergyModelBuilder {
    time_models: HashMap<String, Rc<dyn TraversalModelBuilder>>,
}

impl EnergyModelBuilder {
    pub fn new(time_models: HashMap<String, Rc<dyn TraversalModelBuilder>>) -> EnergyModelBuilder {
        EnergyModelBuilder { time_models }
    }
}

impl TraversalModelBuilder for EnergyModelBuilder {
    fn build(
        &self,
        params: &serde_json::Value,
    ) -> Result<Arc<dyn TraversalModelService>, TraversalModelError> {
        let parent_key = String::from("energy traversal model");

        // load the underlying travel time model
        let time_model_params = params.get("time_model").ok_or_else(|| {
            TraversalModelError::BuildError(
                format!("{} missing time_model parameters", parent_key,),
            )
        })?;
        let time_model_type = time_model_params
            .get_config_string(&"type", &parent_key)
            .map_err(|e| TraversalModelError::BuildError(e.to_string()))?;

        let time_builder = self.time_models.get(&time_model_type).ok_or_else(|| {
            let valid_models = self.time_models.keys().join(",");
            TraversalModelError::BuildError(format!(
                "unknown time_model {}, must be one of [{}]",
                time_model_type, valid_models
            ))
        })?;
        let time_model_service = time_builder.build(time_model_params)?;
        let time_model_speed_unit = time_model_params
            .get_config_serde::<SpeedUnit>(&"speed_unit", &"time_model")
            .map_err(|e| TraversalModelError::BuildError(e.to_string()))?;

        let grade_table_path_option = params
            .get_config_path_optional(&"grade_table_input_file", &parent_key)
            .map_err(|e| TraversalModelError::BuildError(e.to_string()))?;
        let grade_table_grade_unit = params
            .get_config_serde::<GradeUnit>(&"grade_table_grade_unit", &parent_key)
            .map_err(|e| TraversalModelError::BuildError(e.to_string()))?;

        let vehicle_configs = params
            .get_config_array(&"vehicles", &parent_key)
            .map_err(|e| TraversalModelError::BuildError(e.to_string()))?;

        // read all vehicle configurations
        let mut vehicle_library = HashMap::new();
        for vehicle_config in vehicle_configs {
            let vehicle_type = vehicle_config
                .get_config_string(&"type", &parent_key)
                .map_err(|e| TraversalModelError::BuildError(e.to_string()))?;
            let vehicle_builder = VehicleBuilder::from_string(vehicle_type).map_err(|e| {
                TraversalModelError::BuildError(format!("Error building vehicle builder: {}", e))
            })?;
            let vehicle = vehicle_builder
                .build(&vehicle_config)
                .map_err(|e| TraversalModelError::BuildError(e.to_string()))?;
            vehicle_library.insert(vehicle.name(), vehicle);
        }

        let time_unit_option = params
            .get_config_serde_optional::<TimeUnit>(&"time_unit", &parent_key)
            .map_err(|e| TraversalModelError::BuildError(e.to_string()))?;
        let distance_unit_option = params
            .get_config_serde_optional::<DistanceUnit>(&"distance_unit", &parent_key)
            .map_err(|e| TraversalModelError::BuildError(e.to_string()))?;

        let service = EnergyModelService::new(
            time_model_service,
            time_model_speed_unit,
            &grade_table_path_option,
            grade_table_grade_unit,
            time_unit_option,
            distance_unit_option,
            vehicle_library,
        )?;

        Ok(Arc::new(service))
    }
}
