use compass_core::algorithm::search::{edge_traversal::EdgeTraversal, search_error::SearchError};

use crate::{app::search::search_app_result::SearchAppResult, plugin::plugin_error::PluginError};

pub trait OutputPlugin: Send + Sync {
    fn proccess(
        &self,
        output: &serde_json::Value,
        result: Result<&SearchAppResult, SearchError>,
    ) -> Result<serde_json::Value, PluginError>;
}
