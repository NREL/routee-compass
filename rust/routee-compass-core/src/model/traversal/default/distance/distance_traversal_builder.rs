use super::DistanceTraversalService;
use crate::config::CompassConfigurationField;
use crate::config::ConfigJsonExtensions;
use crate::model::traversal::TraversalModelBuilder;
use crate::model::traversal::TraversalModelError;
use crate::model::traversal::TraversalModelService;
use crate::model::unit::baseunit;
use crate::model::unit::DistanceUnit;
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
