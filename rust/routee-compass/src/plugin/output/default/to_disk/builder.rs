use std::{
    fs::OpenOptions,
    path::PathBuf,
    sync::{Arc, Mutex},
};

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

        let output_file_path = PathBuf::from(&output_filename);

        // initialize the file with nothing
        std::fs::write(&output_file_path, "")?;

        // open the file with the option to append to it
        let file = OpenOptions::new()
            .write(true)
            .append(true)
            .open(&output_file_path)
            .map_err(|e| {
                PluginError::FileReadError(
                    output_file_path.clone(),
                    format!("Could not open output file: {}", e),
                )
            })?;

        // wrap the file in a mutex so we can share it between threads
        let output_file = Arc::new(Mutex::new(file));

        let to_disk_plugin = ToDiskOutputPlugin {
            output_file_path,
            output_file,
        };
        Ok(Box::new(to_disk_plugin))
    }
}
