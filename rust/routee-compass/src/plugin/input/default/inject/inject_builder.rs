use std::sync::Arc;

use super::inject_format::InjectFormat;
use crate::{
    app::compass::{CompassConfigurationError, ConfigJsonExtensions},
    plugin::input::{default::inject::InjectInputPlugin, InputPlugin, InputPluginBuilder},
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
        let overwrite: Option<bool> =
            parameters.get_config_serde_optional(&"overwrite", &"inject")?;
        let plugin = InjectInputPlugin::new(key, value, overwrite);
        Ok(Arc::new(plugin))
    }
}
