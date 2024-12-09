use std::sync::Arc;

use crate::{
    app::compass::config::{
        builders::OutputPluginBuilder, compass_configuration_error::CompassConfigurationError,
        config_json_extension::ConfigJsonExtensions,
    },
    plugin::{output::output_plugin::OutputPlugin, plugin_error::PluginError},
};

use super::plugin::UUIDOutputPlugin;

pub struct UUIDOutputPluginBuilder {}

impl OutputPluginBuilder for UUIDOutputPluginBuilder {
    fn build(
        &self,
        parameters: &serde_json::Value,
    ) -> Result<Arc<dyn OutputPlugin>, CompassConfigurationError> {
        let uuid_filename = parameters.get_config_path(&"uuid_input_file", &"uuid")?;

        let uuid_plugin = UUIDOutputPlugin::from_file(&uuid_filename).map_err(|e| {
            let pe = PluginError::OutputPluginFailed { source: e };
            CompassConfigurationError::PluginError(pe)
        })?;
        Ok(Arc::new(uuid_plugin))
    }
}
