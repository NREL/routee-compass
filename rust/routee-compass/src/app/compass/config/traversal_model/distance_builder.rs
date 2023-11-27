use crate::app::compass::config::compass_configuration_field::CompassConfigurationField;
use crate::app::compass::config::config_json_extension::ConfigJsonExtensions;
use routee_compass_core::model::traversal::traversal_model::TraversalModel;
use routee_compass_core::model::traversal::traversal_model_builder::TraversalModelBuilder;
use routee_compass_core::model::traversal::traversal_model_error::TraversalModelError;
use routee_compass_core::model::traversal::traversal_model_service::TraversalModelService;
use routee_compass_core::util::unit::BASE_DISTANCE_UNIT;
use routee_compass_core::{
    model::traversal::default::distance::DistanceModel, util::unit::DistanceUnit,
};
use std::sync::Arc;

pub struct DistanceBuilder {}

pub struct DistanceService {
    distance_unit: DistanceUnit,
}

impl TraversalModelBuilder for DistanceBuilder {
    fn build(
        &self,
        parameters: &serde_json::Value,
    ) -> Result<Arc<dyn TraversalModelService>, TraversalModelError> {
        let traversal_key = CompassConfigurationField::Traversal.to_string();
        let distance_unit_option = parameters
            .get_config_serde_optional::<DistanceUnit>(
                String::from("distance_unit"),
                traversal_key.clone(),
            )
            .map_err(|e| TraversalModelError::BuildError(e.to_string()))?;
        let distance_unit = distance_unit_option.unwrap_or(BASE_DISTANCE_UNIT);
        let m: Arc<dyn TraversalModelService> = Arc::new(DistanceService { distance_unit });
        Ok(m)
    }
}

impl TraversalModelService for DistanceService {
    fn build(
        &self,
        _parameters: &serde_json::Value,
    ) -> Result<Arc<dyn TraversalModel>, TraversalModelError> {
        let m: Arc<dyn TraversalModel> = Arc::new(DistanceModel::new(self.distance_unit));
        Ok(m)
    }
}
