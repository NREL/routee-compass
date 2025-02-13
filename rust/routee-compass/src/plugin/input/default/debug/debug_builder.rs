use super::debug_plugin::DebugInputPlugin;
use crate::plugin::input::{input_plugin::InputPlugin, InputPluginBuilder};
use routee_compass_core::config::CompassConfigurationError;
use std::sync::Arc;

pub struct DebugInputPluginBuilder {}

impl InputPluginBuilder for DebugInputPluginBuilder {
    fn build(
        &self,
        _parameters: &serde_json::Value,
    ) -> Result<Arc<dyn InputPlugin>, CompassConfigurationError> {
        Ok(Arc::new(DebugInputPlugin {}))
    }
}
