use crate::app::{
    compass::compass_app_error::CompassAppError, search::search_app_result::SearchAppResult,
};
use crate::plugin::output::output_plugin::OutputPlugin;
use crate::plugin::plugin_error::PluginError;
use routee_compass_core::algorithm::search::search_instance::SearchInstance;
use routee_compass_core::util::duration_extension::DurationExtension;
use serde_json;

pub struct SummaryOutputPlugin {}

impl OutputPlugin for SummaryOutputPlugin {
    /// append "Cost" value to the output JSON
    fn process(
        &self,
        output: &mut serde_json::Value,
        search_result: &Result<(SearchAppResult, SearchInstance), CompassAppError>,
    ) -> Result<(), PluginError> {
        match search_result {
            Err(_e) => Ok(()),
            Ok((result, _)) => {
                let memory_bytes = allocative::size_of_unique(result) as f64;
                let memory_mb = memory_bytes / 1_024_000.0;
                output["search_executed_time"] = result.search_executed_time.clone().into();
                output["algorithm_runtime"] = result.algorithm_runtime.hhmmss().into();
                output["search_app_runtime"] = result.search_app_runtime.hhmmss().into();
                output["route_size_count"] = result.route.len().into();
                output["tree_size_count"] = result.tree.len().into();
                output["tree_size_mb"] = memory_mb.into();
                output["iterations"] = result.iterations.into();
                Ok(())
            }
        }
    }
}
