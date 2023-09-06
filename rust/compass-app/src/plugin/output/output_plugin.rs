use compass_core::algorithm::search::{edge_traversal::EdgeTraversal, search_error::SearchError};

use crate::plugin::plugin_error::PluginError;

pub trait OutputPlugin: Send + Sync {
    fn proccess(
        &self,
        output: &serde_json::Value,
        search_result: Result<&Vec<EdgeTraversal>, SearchError>,
    ) -> Result<serde_json::Value, PluginError>;
}
