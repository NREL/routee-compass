use std::path::PathBuf;

use crate::{
    app::compass::config::{
        builders::OutputPluginBuilder, compass_configuration_error::CompassConfigurationError,
        config_json_extension::ConfigJsonExtensions,
    },
    plugin::{output::output_plugin::OutputPlugin, plugin_error::PluginError},
};

use super::plugin::ToDiskOutputPlugin;

pub struct ToDiskOutputPluginBuilder {}

impl OutputPluginBuilder for ToDiskOutputPluginBuilder {
    fn build(
        &self,
        parameters: &serde_json::Value,
    ) -> Result<Box<dyn OutputPlugin>, CompassConfigurationError> {
        let output_filename_key = String::from("output_file");
        let output_filename =
            parameters.get_config_string(output_filename_key, String::from("output"))?;

        let output_filepath = PathBuf::from(&output_filename);

        // create empty file
        std::fs::write(output_filepath.clone(), "").map_err(|e| {
            PluginError::FileReadError(
                output_filepath.clone(),
                format!(
                    "Error initializing output file for to_disk output plugin : {}",
                    e
                ),
            )
        })?;

        let to_disk_plugin = ToDiskOutputPlugin {
            output_file: output_filepath,
        };
        Ok(Box::new(to_disk_plugin))
    }
}
