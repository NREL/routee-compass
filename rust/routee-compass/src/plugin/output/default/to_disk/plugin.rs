use std::fs::File;
use std::io::prelude::*;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use crate::{
    app::{
        compass::compass_app_error::CompassAppError, search::search_app_result::SearchAppResult,
    },
    plugin::{output::output_plugin::OutputPlugin, plugin_error::PluginError},
};

pub struct ToDiskOutputPlugin {
    pub output_file_path: PathBuf,
    pub output_file: Arc<Mutex<File>>,
}

impl OutputPlugin for ToDiskOutputPlugin {
    fn process(
        &self,
        output: &serde_json::Value,
        _result: &Result<SearchAppResult, CompassAppError>,
    ) -> Result<Vec<serde_json::Value>, PluginError> {
        let file_ref = Arc::clone(&self.output_file);
        let mut file = file_ref.lock().map_err(|e| {
            PluginError::FileReadError(
                self.output_file_path.clone(),
                format!("Could not aquire lock on output file: {}", e),
            )
        })?;

        let output_json = serde_json::to_string(output)?;

        writeln!(file, "{}", output_json).map_err(|e| {
            PluginError::FileReadError(
                self.output_file_path.clone(),
                format!("Could not write to output file: {}", e),
            )
        })?;

        // return empty vec since we already wrote the result to a file
        Ok(Vec::new())
    }
}
