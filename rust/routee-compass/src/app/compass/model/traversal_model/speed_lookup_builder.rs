use routee_compass_core::config::CompassConfigurationField;
use routee_compass_core::config::ConfigJsonExtensions;
use routee_compass_core::model::traversal::default::SpeedLookupService;
use routee_compass_core::model::traversal::default::SpeedTraversalEngine;
use routee_compass_core::model::traversal::TraversalModelBuilder;
use routee_compass_core::model::traversal::TraversalModelError;
use routee_compass_core::model::traversal::TraversalModelService;
use routee_compass_core::model::unit::{DistanceUnit, SpeedUnit, TimeUnit};
use std::sync::Arc;

pub struct SpeedLookupBuilder {}

impl TraversalModelBuilder for SpeedLookupBuilder {
    fn build(
        &self,
        params: &serde_json::Value,
    ) -> Result<Arc<dyn TraversalModelService>, TraversalModelError> {
        let traversal_key = CompassConfigurationField::Traversal.to_string();
        // todo: optional output time unit
        let filename = params
            .get_config_path(&"speed_table_input_file", &traversal_key)
            .map_err(|e| TraversalModelError::BuildError(e.to_string()))?;
        let speed_unit = params
            .get_config_serde::<SpeedUnit>(&"speed_unit", &traversal_key)
            .map_err(|e| TraversalModelError::BuildError(e.to_string()))?;
        let distance_unit = params
            .get_config_serde_optional::<DistanceUnit>(&"distance_unit", &traversal_key)
            .map_err(|e| TraversalModelError::BuildError(e.to_string()))?;
        let time_unit = params
            .get_config_serde_optional::<TimeUnit>(&"time_unit", &traversal_key)
            .map_err(|e| TraversalModelError::BuildError(e.to_string()))?;

        let e = SpeedTraversalEngine::new(&filename, speed_unit, distance_unit, time_unit)?;
        let service = Arc::new(SpeedLookupService { e: Arc::new(e) });
        Ok(service)
    }
}
