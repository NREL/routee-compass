use super::plugin::UUIDOutputPlugin;
use crate::{
    app::compass::{CompassConfigurationError, ConfigJsonExtensions},
    plugin::{
        output::{OutputPlugin, OutputPluginBuilder},
        PluginError,
    },
};
use std::sync::Arc;

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
