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
    ) -> Result<Box<dyn InputPlugin>, CompassConfigurationError> {
        let key = parameters.get_config_string(String::from("key"), String::from("inject"))?;
        let value_string =
            parameters.get_config_string(String::from("value"), String::from("inject"))?;
        let format: InjectFormat =
            parameters.get_config_serde(String::from("format"), String::from("inject"))?;
        let value = format.to_json(&value_string)?;
        let plugin = InjectInputPlugin::new(key, value);
        Ok(Box::new(plugin))
    }
}
