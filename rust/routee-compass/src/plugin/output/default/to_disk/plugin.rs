use std::io::prelude::*;
use std::{fs::OpenOptions, path::PathBuf};

use crate::{
    app::{
        compass::compass_app_error::CompassAppError, search::search_app_result::SearchAppResult,
    },
    plugin::{output::output_plugin::OutputPlugin, plugin_error::PluginError},
};

pub struct ToDiskOutputPlugin {
    pub output_file: PathBuf,
}

impl OutputPlugin for ToDiskOutputPlugin {
    fn process(
        &self,
        output: &serde_json::Value,
        _result: &Result<SearchAppResult, CompassAppError>,
    ) -> Result<Vec<serde_json::Value>, PluginError> {
        let mut file = OpenOptions::new()
            .write(true)
            .append(true)
            .open(&self.output_file)
            .map_err(|e| {
                PluginError::FileReadError(
                    self.output_file.clone(),
                    format!("Could not open output file: {}", e),
                )
            })?;
        let output_json = serde_json::to_string(output)?;

        writeln!(file, "{}", output_json).map_err(|e| {
            PluginError::FileReadError(
                self.output_file.clone(),
                format!("Could not write to output file: {}", e),
            )
        })?;

        // return empty vec since we already wrote the result to a file
        Ok(Vec::new())
    }
}
