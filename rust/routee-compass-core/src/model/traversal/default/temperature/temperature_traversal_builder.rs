use crate::model::traversal::{
    default::temperature::{
        ambient_temperature_config::AmbientTemperatureConfig, TemperatureTraversalService,
    },
    TraversalModelBuilder, TraversalModelError, TraversalModelService,
};
use std::sync::Arc;

pub struct TemperatureTraversalBuilder {}

impl TraversalModelBuilder for TemperatureTraversalBuilder {
    fn build(
        &self,
        parameters: &serde_json::Value,
    ) -> Result<Arc<dyn TraversalModelService>, TraversalModelError> {
        let ambient_temp_config: Option<AmbientTemperatureConfig> = parameters
            .get("default_ambient_temperature")
            .map(|v| serde_json::from_value(v.clone()).map_err(|e| TraversalModelError::BuildError(
                format!("Attempted to parse the default_ambient_temperature key from the config but failed. Expected a json object with a value and a unit key but got this error: {e}")))
            ).transpose()?;

        let service = Arc::new(TemperatureTraversalService {
            default_ambient_temperature: ambient_temp_config,
        });
        Ok(service)
    }
}
