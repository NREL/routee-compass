use crate::app::compass::config::compass_configuration_field::CompassConfigurationField;
use crate::app::compass::config::config_json_extension::ConfigJsonExtensions;
use routee_compass_core::model::traversal::default::speed_lookup_model::SpeedLookupModel;
use routee_compass_core::model::traversal::traversal_model::TraversalModel;
use routee_compass_core::model::traversal::traversal_model_builder::TraversalModelBuilder;
use routee_compass_core::model::traversal::traversal_model_error::TraversalModelError;
use routee_compass_core::model::traversal::traversal_model_service::TraversalModelService;
use routee_compass_core::util::unit::{DistanceUnit, SpeedUnit, TimeUnit};
use std::sync::Arc;

pub struct SpeedLookupBuilder {}

pub struct SpeedLookupService {
    m: Arc<SpeedLookupModel>,
}

impl TraversalModelBuilder for SpeedLookupBuilder {
    fn build(
        &self,
        params: &serde_json::Value,
    ) -> Result<Arc<dyn TraversalModelService>, TraversalModelError> {
        let traversal_key = CompassConfigurationField::Traversal.to_string();
        // todo: optional output time unit
        let filename = params
            .get_config_path(
                String::from("speed_table_input_file"),
                traversal_key.clone(),
            )
            .map_err(|e| TraversalModelError::BuildError(e.to_string()))?;
        let speed_unit = params
            .get_config_serde::<SpeedUnit>(String::from("speed_unit"), traversal_key.clone())
            .map_err(|e| TraversalModelError::BuildError(e.to_string()))?;
        let distance_unit = params
            .get_config_serde_optional::<DistanceUnit>(
                String::from("output_distance_unit"),
                traversal_key.clone(),
            )
            .map_err(|e| TraversalModelError::BuildError(e.to_string()))?;
        let time_unit = params
            .get_config_serde_optional::<TimeUnit>(
                String::from("output_time_unit"),
                traversal_key.clone(),
            )
            .map_err(|e| TraversalModelError::BuildError(e.to_string()))?;

        let m = SpeedLookupModel::new(&filename, speed_unit, distance_unit, time_unit)?;
        let service = Arc::new(SpeedLookupService { m: Arc::new(m) });
        Ok(service)
    }
}

impl TraversalModelService for SpeedLookupService {
    fn build(
        &self,
        _parameters: &serde_json::Value,
    ) -> Result<Arc<dyn TraversalModel>, TraversalModelError> {
        Ok(self.m.clone())
    }
}
