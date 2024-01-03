use std::sync::Arc;

use super::inject_format::InjectFormat;
use crate::{
    app::compass::config::{
        builders::InputPluginBuilder, compass_configuration_error::CompassConfigurationError,
        config_json_extension::ConfigJsonExtensions,
    },
    plugin::input::{default::inject::inject_plugin::InjectInputPlugin, input_plugin::InputPlugin},
};

pub struct InjectPluginBuilder {}

impl InputPluginBuilder for InjectPluginBuilder {
    fn build(
        &self,
        parameters: &serde_json::Value,
    ) -> Result<Arc<dyn InputPlugin>, CompassConfigurationError> {
        let key = parameters.get_config_string(&"key", &"inject")?;
        let value_string = parameters.get_config_string(&"value", &"inject")?;
        let format: InjectFormat = parameters.get_config_serde(&"format", &"inject")?;
        let value = format.to_json(&value_string)?;
        let plugin = InjectInputPlugin::new(key, value);
        Ok(Arc::new(plugin))
    }
}
