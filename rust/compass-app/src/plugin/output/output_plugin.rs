use crate::plugin::plugin_error::PluginError;
use compass_core::algorithm::search::search_algorithm_result::SearchAlgorithmResult;
use compass_core::algorithm::search::search_error::SearchError;

pub trait OutputPlugin: Send + Sync {
    fn process(
        &self,
        output: &serde_json::Value,
        result: Result<&SearchAlgorithmResult, SearchError>,
    ) -> Result<serde_json::Value, PluginError>;
}
