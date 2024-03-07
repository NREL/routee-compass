use crate::app::compass::compass_app_error::CompassAppError;
use crate::app::search::search_app_result::SearchAppResult;
use crate::plugin::output::default::edgeidlist::json_extensions::EdgeListJsonExtensions;
use crate::plugin::output::output_plugin::OutputPlugin;
use crate::plugin::plugin_error::PluginError;
use routee_compass_core::algorithm::search::search_instance::SearchInstance;
use serde_json;

pub struct EdgeIdListOutputPlugin {}

impl OutputPlugin for EdgeIdListOutputPlugin {
    fn process(
        &self,
        output: &mut serde_json::Value,
        search_result: &Result<(SearchAppResult, SearchInstance), CompassAppError>,
    ) -> Result<(), PluginError> {
        match search_result {
            Err(_e) => Ok(()),
            Ok((result, _)) => {
                let edge_ids = result
                    .route
                    .clone()
                    .iter()
                    .map(|e| e.edge_id)
                    .collect::<Vec<_>>();
                output.add_edge_list(&edge_ids)?;
                Ok(())
            }
        }
    }
}
