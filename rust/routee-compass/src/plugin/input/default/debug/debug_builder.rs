use crate::{
    app::compass::config::{
        builders::InputPluginBuilder, compass_configuration_error::CompassConfigurationError,
    },
    plugin::input::input_plugin::InputPlugin,
};
use std::sync::Arc;

use super::debug_plugin::DebugInputPlugin;

pub struct DebugInputPluginBuilder {}

impl InputPluginBuilder for DebugInputPluginBuilder {
    fn build(
        &self,
        _parameters: &serde_json::Value,
    ) -> Result<Arc<dyn InputPlugin>, CompassConfigurationError> {
        Ok(Arc::new(DebugInputPlugin {}))
    }
}
