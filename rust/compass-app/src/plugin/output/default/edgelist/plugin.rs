use crate::plugin::plugin_error::PluginError;

use compass_core::{
    algorithm::search::{edge_traversal::EdgeTraversal, search_error::SearchError},
    model::cost::cost::Cost,
};
use serde_json;

use crate::plugin::output::output_plugin::OutputPlugin;

use super::json_extensions::EdgeListJsonExtensions;

pub struct EdgeListOutputPlugin {}

impl OutputPlugin for EdgeListOutputPlugin {
    fn proccess(
        &self,
        output: &serde_json::Value,
        search_result: Result<&Vec<EdgeTraversal>, SearchError>,
    ) -> Result<serde_json::Value, PluginError> {
        todo!()
    }
}

#[cfg(test)]

mod tests {

    use super::*;
}
