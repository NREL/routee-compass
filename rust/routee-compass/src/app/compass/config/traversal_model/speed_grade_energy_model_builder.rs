use std::sync::Arc;

use crate::app::compass::compass_configuration_field::CompassConfigurationField;
use crate::app::compass::config::builders::TraversalModelService;
use crate::app::compass::config::config_json_extension::ConfigJsonExtensions;
use crate::app::compass::config::{
    builders::TraversalModelBuilder, compass_configuration_error::CompassConfigurationError,
};
use routee_compass_core::model::traversal::traversal_model::TraversalModel;
use routee_compass_core::util::unit::{
    DistanceUnit, EnergyRate, EnergyRateUnit, GradeUnit, SpeedUnit, TimeUnit,
};
use routee_compass_powertrain::routee::model_type::ModelType;
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

        let speed_table_path =
            params.get_config_string(String::from("speed_table_path"), traversal_key.clone())?;
        let speed_table_speed_unit = params.get_config_serde::<SpeedUnit>(
            String::from("speed_table_speed_unit"),
            traversal_key.clone(),
        )?;

        let grade_table_path = params.get_config_serde_optional::<String>(
            String::from("grade_table_path"),
            traversal_key.clone(),
        )?;
        let grade_table_grade_unit = params.get_config_serde_optional::<GradeUnit>(
            String::from("graph_grade_unit"),
            traversal_key.clone(),
        )?;

        let energy_model_path =
            params.get_config_string(String::from("energy_model_path"), traversal_key.clone())?;
        let model_type = params
            .get_config_serde::<ModelType>(String::from("model_type"), traversal_key.clone())?;
        let energy_model_speed_unit = params.get_config_serde::<SpeedUnit>(
            String::from("energy_model_speed_unit"),
            traversal_key.clone(),
        )?;
        let ideal_energy_rate_option = params.get_config_serde_optional::<EnergyRate>(
            String::from("ideal_energy_rate"),
            traversal_key.clone(),
        )?;
        let energy_model_grade_unit = params.get_config_serde::<GradeUnit>(
            String::from("energy_model_grade_unit"),
            traversal_key.clone(),
        )?;

        let energy_model_energy_rate_unit = params.get_config_serde::<EnergyRateUnit>(
            String::from("energy_model_energy_rate_unit"),
            traversal_key.clone(),
        )?;

        let output_time_unit_option = params.get_config_serde_optional::<TimeUnit>(
            String::from("output_time_unit"),
            traversal_key.clone(),
        )?;
        let output_distance_unit_option = params.get_config_serde_optional::<DistanceUnit>(
            String::from("output_distance_unit"),
            traversal_key.clone(),
        )?;

        let real_world_energy_adjustment = params.get_config_serde_optional::<f64>(
            String::from("real_world_energy_adjustment"),
            traversal_key.clone(),
        )?;

        let inner_service = SpeedGradeModelService::new(
            speed_table_path,
            speed_table_speed_unit,
            grade_table_path,
            grade_table_grade_unit,
            energy_model_path,
            model_type,
            ideal_energy_rate_option,
            energy_model_speed_unit,
            energy_model_grade_unit,
            energy_model_energy_rate_unit,
            output_time_unit_option,
            output_distance_unit_option,
            real_world_energy_adjustment,
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
