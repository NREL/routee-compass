use crate::app::search::search_app_result::SearchAppResult;
use crate::plugin::output::default::edgeidlist::json_extensions::EdgeListJsonExtensions;
use crate::plugin::output::output_plugin::OutputPlugin;
use crate::plugin::plugin_error::PluginError;
use routee_compass_core::algorithm::search::search_error::SearchError;
use serde_json;

pub struct EdgeIdListOutputPlugin {}

impl OutputPlugin for EdgeIdListOutputPlugin {
    fn process(
        &self,
        output: &serde_json::Value,
        search_result: Result<&SearchAppResult, SearchError>,
    ) -> Result<serde_json::Value, PluginError> {
        match search_result {
            Err(_e) => Ok(output.clone()),
            Ok(result) => {
                let edge_ids = result
                    .route
                    .clone()
                    .iter()
                    .map(|e| e.edge_id)
                    .collect::<Vec<_>>();
                let mut updated = output.clone();
                updated.add_edge_list(&edge_ids)?;
                Ok(updated)
            }
        }
    }
}
