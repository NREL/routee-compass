use std::collections::HashMap;
use std::sync::Arc;

use crate::app::compass::config::builders::TraversalModelService;
use crate::app::compass::config::compass_configuration_field::CompassConfigurationField;
use crate::app::compass::config::config_json_extension::ConfigJsonExtensions;
use crate::app::compass::config::{
    builders::TraversalModelBuilder, compass_configuration_error::CompassConfigurationError,
};
use routee_compass_core::model::traversal::traversal_model::TraversalModel;
use routee_compass_core::util::unit::{DistanceUnit, GradeUnit, SpeedUnit, TimeUnit};
use routee_compass_powertrain::routee::energy_model_service::EnergyModelService;
use routee_compass_powertrain::routee::energy_traversal_model::EnergyTraversalModel;

use super::energy_model_vehicle_builders::{
    ConventionalVehicleBuilder, PlugInHybridBuilder, VehicleBuilder,
};

pub struct EnergyModelBuilder {
    pub vehicle_builders: HashMap<String, Box<dyn VehicleBuilder>>,
}

impl TraversalModelBuilder for EnergyModelBuilder {
    fn build(
        &self,
        params: &serde_json::Value,
    ) -> Result<Arc<dyn TraversalModelService>, CompassConfigurationError> {
        let traversal_key = CompassConfigurationField::Traversal.to_string();

        let speed_table_path = params.get_config_path(
            String::from("speed_table_input_file"),
            traversal_key.clone(),
        )?;
        let speed_table_speed_unit = params.get_config_serde::<SpeedUnit>(
            String::from("speed_table_speed_unit"),
            traversal_key.clone(),
        )?;

        let grade_table_path = params.get_config_path_optional(
            String::from("grade_table_input_file"),
            traversal_key.clone(),
        )?;
        let grade_table_grade_unit = params.get_config_serde_optional::<GradeUnit>(
            String::from("graph_grade_unit"),
            traversal_key.clone(),
        )?;

        let vehicle_configs =
            params.get_config_array("vehicles".to_string(), traversal_key.clone())?;

        let mut vehicle_library = HashMap::new();

        for vehicle_config in vehicle_configs {
            let vehicle_type =
                vehicle_config.get_config_string(String::from("type"), traversal_key.clone())?;
            let vehicle_builder = self.vehicle_builders.get(&vehicle_type).ok_or(
                CompassConfigurationError::UnknownModelNameForComponent(
                    vehicle_type.clone(),
                    "vehicle".to_string(),
                ),
            )?;
            let vehicle = vehicle_builder.build(&vehicle_config)?;
            vehicle_library.insert(vehicle.name(), vehicle);
        }

        let output_time_unit_option = params.get_config_serde_optional::<TimeUnit>(
            String::from("output_time_unit"),
            traversal_key.clone(),
        )?;
        let output_distance_unit_option = params.get_config_serde_optional::<DistanceUnit>(
            String::from("output_distance_unit"),
            traversal_key.clone(),
        )?;

        let service = EnergyModelService::new(
            &speed_table_path,
            speed_table_speed_unit,
            &grade_table_path,
            grade_table_grade_unit,
            output_time_unit_option,
            output_distance_unit_option,
            vehicle_library,
        )
        .map_err(CompassConfigurationError::TraversalModelError)?;

        Ok(Arc::new(service))
    }
}

impl TraversalModelService for EnergyModelService {
    fn build(
        &self,
        parameters: &serde_json::Value,
    ) -> Result<Arc<dyn TraversalModel>, CompassConfigurationError> {
        let arc_self = Arc::new(self.clone());
        let model = EnergyTraversalModel::try_from((arc_self, parameters))?;
        Ok(Arc::new(model))
    }
}

impl Default for EnergyModelBuilder {
    fn default() -> Self {
        let mut vehicle_builders: HashMap<String, Box<dyn VehicleBuilder>> = HashMap::new();
        vehicle_builders.insert(
            "conventional".to_string(),
            Box::new(ConventionalVehicleBuilder {}),
        );
        vehicle_builders.insert(
            "plug_in_hybrid".to_string(),
            Box::new(PlugInHybridBuilder {}),
        );
        Self { vehicle_builders }
    }
}
