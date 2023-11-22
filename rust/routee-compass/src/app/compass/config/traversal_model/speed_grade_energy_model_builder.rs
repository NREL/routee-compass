use std::collections::HashMap;
use std::sync::Arc;

use crate::app::compass::config::compass_configuration_error::CompassConfigurationError;
use crate::app::compass::config::compass_configuration_field::CompassConfigurationField;
use crate::app::compass::config::config_json_extension::ConfigJsonExtensions;
use routee_compass_core::model::traversal::traversal_model_builder::TraversalModelBuilder;
use routee_compass_core::model::traversal::traversal_model_error::TraversalModelError;
use routee_compass_core::model::traversal::traversal_model_service::TraversalModelService;
use routee_compass_core::util::unit::{
    DistanceUnit, EnergyRate, EnergyRateUnit, GradeUnit, SpeedUnit, TimeUnit,
};
use routee_compass_powertrain::routee::model_type::ModelType;
use routee_compass_powertrain::routee::prediction_model::SpeedGradePredictionModelRecord;
use routee_compass_powertrain::routee::speed_grade_energy_model_service::SpeedGradeEnergyModelService;

pub struct SpeedGradeEnergyModelBuilder {}

// pub struct SpeedGradeEnergyModelService {
//     service: SpeedGradeModelService,
// }

impl TraversalModelBuilder for SpeedGradeEnergyModelBuilder {
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

        let energy_model_configs = params
            .get_config_array("energy_models".to_string(), traversal_key.clone())
            .map_err(|e| TraversalModelError::BuildError(e.to_string()))?;

        let mut energy_model_library = HashMap::new();

        for energy_model_config in energy_model_configs {
            let name = energy_model_config
                .get_config_string(String::from("name"), traversal_key.clone())
                .map_err(|e| TraversalModelError::BuildError(e.to_string()))?;
            let model_path = energy_model_config
                .get_config_path(String::from("model_input_file"), traversal_key.clone())
                .map_err(|e| TraversalModelError::BuildError(e.to_string()))?;
            let model_type = energy_model_config
                .get_config_serde::<ModelType>(String::from("model_type"), traversal_key.clone())
                .map_err(|e| TraversalModelError::BuildError(e.to_string()))?;
            let speed_unit = energy_model_config
                .get_config_serde::<SpeedUnit>(String::from("speed_unit"), traversal_key.clone())
                .map_err(|e| TraversalModelError::BuildError(e.to_string()))?;
            let ideal_energy_rate_option = energy_model_config
                .get_config_serde_optional::<EnergyRate>(
                    String::from("ideal_energy_rate"),
                    traversal_key.clone(),
                )
                .map_err(|e| TraversalModelError::BuildError(e.to_string()))?;
            let grade_unit = energy_model_config
                .get_config_serde::<GradeUnit>(String::from("grade_unit"), traversal_key.clone())
                .map_err(|e| TraversalModelError::BuildError(e.to_string()))?;

            let energy_rate_unit = energy_model_config
                .get_config_serde::<EnergyRateUnit>(
                    String::from("energy_rate_unit"),
                    traversal_key.clone(),
                )
                .map_err(|e| TraversalModelError::BuildError(e.to_string()))?;
            let real_world_energy_adjustment_option = params
                .get_config_serde_optional::<f64>(
                    String::from("real_world_energy_adjustment"),
                    traversal_key.clone(),
                )
                .map_err(|e| TraversalModelError::BuildError(e.to_string()))?;

            let model_record = SpeedGradePredictionModelRecord::new(
                name.clone(),
                &model_path,
                model_type,
                speed_unit,
                grade_unit,
                energy_rate_unit,
                ideal_energy_rate_option,
                real_world_energy_adjustment_option,
            )
            .map_err(|e| TraversalModelError::BuildError(e.to_string()))?;
            energy_model_library.insert(name, Arc::new(model_record));
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

        let service = SpeedGradeEnergyModelService::new(
            &speed_table_path,
            speed_table_speed_unit,
            &grade_table_path,
            grade_table_grade_unit,
            output_time_unit_option,
            output_distance_unit_option,
            energy_model_library,
        )
        .map_err(CompassConfigurationError::TraversalModelError)
        .map_err(|e| TraversalModelError::BuildError(e.to_string()))?;
        // let service = SpeedGradeEnergyModelService {
        //     service: inner_service,
        // };
        let result: Arc<dyn TraversalModelService> = Arc::new(service);
        Ok(result)
    }
}

// impl TraversalModelService for SpeedGradeEnergyModelService {
//     fn build(
//         &self,
//         parameters: &serde_json::Value,
//     ) -> Result<Arc<dyn TraversalModel>, CompassConfigurationError> {
//         let arc_self = Arc::new(self.service.clone());
//         let m = SpeedGradeModel::try_from((arc_self, parameters)).map_err(|e| TraversalModelError::BuildError(e.to_string()))?;
//         Ok(Arc::new(m))
//     }
// }
