use super::inject_plugin_config::InjectPluginConfig;
use crate::plugin::input::{InputPlugin, InputPluginBuilder};
use routee_compass_core::config::CompassConfigurationError;
use std::sync::Arc;

pub struct InjectPluginBuilder {}

impl InputPluginBuilder for InjectPluginBuilder {
    fn build(
        &self,
        parameters: &serde_json::Value,
    ) -> Result<Arc<dyn InputPlugin>, CompassConfigurationError> {
        let config: InjectPluginConfig = serde_json::from_value(parameters.clone())?;
        let plugin = config.build().map_err(|e| {
            CompassConfigurationError::UserConfigurationError(format!(
                "failed to build inject plugin from configuration: {e}"
            ))
        })?;
        Ok(Arc::new(plugin))
    }
}
