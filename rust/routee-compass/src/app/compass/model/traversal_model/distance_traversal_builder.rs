use routee_compass_core::config::CompassConfigurationField;
use routee_compass_core::config::ConfigJsonExtensions;
use routee_compass_core::model::traversal::default::DistanceTraversalService;
use routee_compass_core::model::traversal::TraversalModelBuilder;
use routee_compass_core::model::traversal::TraversalModelError;
use routee_compass_core::model::traversal::TraversalModelService;
use routee_compass_core::model::unit::baseunit;
use routee_compass_core::model::unit::DistanceUnit;
use std::sync::Arc;

pub struct DistanceTraversalBuilder {}

impl TraversalModelBuilder for DistanceTraversalBuilder {
    fn build(
        &self,
        parameters: &serde_json::Value,
    ) -> Result<Arc<dyn TraversalModelService>, TraversalModelError> {
        let traversal_key = CompassConfigurationField::Traversal.to_string();
        let distance_unit_option = parameters
            .get_config_serde_optional::<DistanceUnit>(&"distance_unit", &traversal_key)
            .map_err(|e| TraversalModelError::BuildError(e.to_string()))?;
        let distance_unit = distance_unit_option.unwrap_or(baseunit::DISTANCE_UNIT);
        let m: Arc<dyn TraversalModelService> =
            Arc::new(DistanceTraversalService { distance_unit });
        Ok(m)
    }
}
