use std::collections::HashMap;
use std::sync::Arc;

use crate::app::compass::config::builders::TraversalModelService;
use crate::app::compass::config::compass_configuration_field::CompassConfigurationField;
use crate::app::compass::config::config_json_extension::ConfigJsonExtensions;
use crate::app::compass::config::{
    builders::TraversalModelBuilder, compass_configuration_error::CompassConfigurationError,
};
use routee_compass_core::model::traversal::traversal_model::TraversalModel;
use routee_compass_core::util::unit::{
    DistanceUnit, EnergyRate, EnergyRateUnit, GradeUnit, SpeedUnit, TimeUnit,
};
use routee_compass_powertrain::routee::model_type::ModelType;
use routee_compass_powertrain::routee::prediction_model::SpeedGradePredictionModelRecord;
use routee_compass_powertrain::routee::speed_grade_model::SpeedGradeModel;
use routee_compass_powertrain::routee::speed_grade_model_service::SpeedGradeModelService;

pub struct SpeedGradeEnergyModelBuilder {}

pub struct SpeedGradeEnergyModelService {
    service: SpeedGradeModelService,
}

impl TraversalModelBuilder for SpeedGradeEnergyModelBuilder {
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

        let energy_model_configs =
            params.get_config_array("energy_models".to_string(), traversal_key.clone())?;

        let mut energy_model_library = HashMap::new();

        for energy_model_config in energy_model_configs {
            let name = energy_model_config
                .get_config_string(String::from("name"), traversal_key.clone())?;
            let model_path = energy_model_config
                .get_config_path(String::from("model_input_file"), traversal_key.clone())?;
            let model_type = energy_model_config
                .get_config_serde::<ModelType>(String::from("model_type"), traversal_key.clone())?;
            let speed_unit = energy_model_config
                .get_config_serde::<SpeedUnit>(String::from("speed_unit"), traversal_key.clone())?;
            let ideal_energy_rate_option = energy_model_config
                .get_config_serde_optional::<EnergyRate>(
                    String::from("ideal_energy_rate"),
                    traversal_key.clone(),
                )?;
            let grade_unit = energy_model_config
                .get_config_serde::<GradeUnit>(String::from("grade_unit"), traversal_key.clone())?;

            let energy_rate_unit = energy_model_config.get_config_serde::<EnergyRateUnit>(
                String::from("energy_rate_unit"),
                traversal_key.clone(),
            )?;
            let real_world_energy_adjustment_option = params.get_config_serde_optional::<f64>(
                String::from("real_world_energy_adjustment"),
                traversal_key.clone(),
            )?;

            let model_record = SpeedGradePredictionModelRecord::new(
                name.clone(),
                &model_path,
                model_type,
                speed_unit,
                grade_unit,
                energy_rate_unit,
                ideal_energy_rate_option,
                real_world_energy_adjustment_option,
            )?;
            energy_model_library.insert(name, Arc::new(model_record));
        }

        let output_time_unit_option = params.get_config_serde_optional::<TimeUnit>(
            String::from("output_time_unit"),
            traversal_key.clone(),
        )?;
        let output_distance_unit_option = params.get_config_serde_optional::<DistanceUnit>(
            String::from("output_distance_unit"),
            traversal_key.clone(),
        )?;

        let inner_service = SpeedGradeModelService::new(
            &speed_table_path,
            speed_table_speed_unit,
            &grade_table_path,
            grade_table_grade_unit,
            output_time_unit_option,
            output_distance_unit_option,
            energy_model_library,
        )
        .map_err(CompassConfigurationError::TraversalModelError)?;
        let service = SpeedGradeEnergyModelService {
            service: inner_service,
        };

        return Ok(Arc::new(service));
    }
}

impl TraversalModelService for SpeedGradeEnergyModelService {
    fn build(
        &self,
        parameters: &serde_json::Value,
    ) -> Result<Arc<dyn TraversalModel>, CompassConfigurationError> {
        let arc_self = Arc::new(self.service.clone());
        let m = SpeedGradeModel::try_from((arc_self, parameters))?;
        Ok(Arc::new(m))
    }
}
