use super::SpeedLookupService;
use super::SpeedTraversalEngine;
use crate::config::CompassConfigurationField;
use crate::config::ConfigJsonExtensions;
use crate::model::traversal::TraversalModelBuilder;
use crate::model::traversal::TraversalModelError;
use crate::model::traversal::TraversalModelService;
use crate::model::unit::SpeedUnit;
use std::sync::Arc;

pub struct SpeedTraversalBuilder {}

impl TraversalModelBuilder for SpeedTraversalBuilder {
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

        let e = SpeedTraversalEngine::new(&filename, speed_unit)?;
        let service = Arc::new(SpeedLookupService { e: Arc::new(e) });
        Ok(service)
    }
}
