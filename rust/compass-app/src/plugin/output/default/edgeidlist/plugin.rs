use crate::plugin::output::default::edgeidlist::json_extensions::EdgeListJsonExtensions;
use crate::plugin::output::output_plugin::OutputPlugin;
use crate::plugin::plugin_error::PluginError;
use compass_core::algorithm::search::{edge_traversal::EdgeTraversal, search_error::SearchError};
use serde_json;

pub struct EdgeIdListOutputPlugin {}

impl OutputPlugin for EdgeIdListOutputPlugin {
    fn proccess(
        &self,
        output: &serde_json::Value,
        search_result: Result<&Vec<EdgeTraversal>, SearchError>,
    ) -> Result<serde_json::Value, PluginError> {
        match search_result {
            Err(_e) => Ok(output.clone()),
            Ok(r) => {
                let edge_ids = r.clone().iter().map(|e| e.edge_id).collect::<Vec<_>>();
                let mut updated = output.clone();
                updated.add_edge_list(&edge_ids)?;
                Ok(updated)
            }
        }
    }
}
