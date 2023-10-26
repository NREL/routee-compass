use crate::{
    app::compass::config::{
        builders::OutputPluginBuilder, compass_configuration_error::CompassConfigurationError,
    },
    plugin::output::output_plugin::OutputPlugin,
};

use super::plugin::UUIDOutputPlugin;

pub struct UUIDOutputPluginBuilder {}

impl OutputPluginBuilder for UUIDOutputPluginBuilder {
    fn build(
        &self,
        parameters: &serde_json::Value,
    ) -> Result<Box<dyn OutputPlugin>, CompassConfigurationError> {
        let uuid_filename_key = String::from("uuid_file");
        let uuid_filename = parameters
            .get(&uuid_filename_key)
            .ok_or(CompassConfigurationError::ExpectedFieldForComponent(
                uuid_filename_key.clone(),
                String::from("UUID Output Plugin"),
            ))?
            .as_str()
            .map(String::from)
            .ok_or(CompassConfigurationError::ExpectedFieldWithType(
                uuid_filename_key.clone(),
                String::from("String"),
            ))?;

        let uuid_plugin = UUIDOutputPlugin::from_file(&uuid_filename)
            .map_err(CompassConfigurationError::PluginError)?;
        Ok(Box::new(uuid_plugin))
    }
}
