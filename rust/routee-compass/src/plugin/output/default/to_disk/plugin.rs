use std::path::PathBuf;

use routee_compass_core::algorithm::search::search_error::SearchError;

use crate::{
    app::{search::search_app_result::SearchAppResult, compass::compass_app_error::CompassAppError},
    plugin::{output::output_plugin::OutputPlugin, plugin_error::PluginError},
};

pub struct ToDiskOutputPlugin {
    output_file: PathBuf,
}

impl OutputPlugin for ToDiskOutputPlugin {
    fn process(
        &self,
        output: &serde_json::Value,
        _result: &Result<SearchAppResult, CompassAppError>,
    ) -> Result<Vec<serde_json::Value>, PluginError> {
        Ok(Vec::new())
    }
}
