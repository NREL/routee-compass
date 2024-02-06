use crate::app::{
    compass::compass_app_error::CompassAppError, search::search_app_result::SearchAppResult,
};
use crate::plugin::output::output_plugin::OutputPlugin;
use crate::plugin::plugin_error::PluginError;
use routee_compass_core::util::duration_extension::DurationExtension;
use serde_json;

pub struct SummaryOutputPlugin {}

impl OutputPlugin for SummaryOutputPlugin {
    /// append "Cost" value to the output JSON
    fn process(
        &self,
        output: &serde_json::Value,
        search_result: &Result<SearchAppResult, CompassAppError>,
    ) -> Result<Vec<serde_json::Value>, PluginError> {
        match search_result {
            Err(_e) => Ok(vec![output.clone()]),
            Ok(result) => {
                let mut updated_output = output.clone();
                let updated = updated_output.as_object_mut().ok_or_else(|| {
                    PluginError::InternalError(format!(
                        "expected output JSON to be an object, found {}",
                        output
                    ))
                })?;
                let memory_usage = allocative::size_of_unique(result) as f64;
                updated.insert("result_memory_usage_bytes".to_string(), memory_usage.into());

                updated.insert(
                    "search_executed_time".to_string(),
                    result.search_start_time.clone().into(),
                );

                updated.insert(
                    "search_runtime".to_string(),
                    result.search_runtime.hhmmss().into(),
                );

                updated.insert(
                    "total_runtime".to_string(),
                    result.total_runtime.hhmmss().into(),
                );

                updated.insert("route_edge_count".to_string(), result.route.len().into());

                let tree_len = match &result.tree {
                    None => 0,
                    Some(tree) => tree.len(),
                };

                updated.insert("tree_edge_count".to_string(), tree_len.into());

                Ok(vec![updated_output])
            }
        }
    }
}
