use crate::app::search::search_app_result::SearchAppResult;
use crate::plugin::plugin_error::PluginError;
use compass_core::algorithm::search::search_error::SearchError;

pub trait OutputPlugin: Send + Sync {
    fn process(
        &self,
        output: &serde_json::Value,
        result: Result<&SearchAppResult, SearchError>,
    ) -> Result<serde_json::Value, PluginError>;
}
