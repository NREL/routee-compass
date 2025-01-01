use super::debug_plugin::DebugInputPlugin;
use crate::{
    app::compass::model::builders::InputPluginBuilder, app::compass::CompassConfigurationError,
    plugin::input::input_plugin::InputPlugin,
};
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
