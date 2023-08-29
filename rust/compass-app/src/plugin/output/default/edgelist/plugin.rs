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
    use compass_core::model::{graph::edge_id::EdgeId, traversal::state::state_variable::StateVar};

    use super::*;

    #[test]
    fn test_EdgeList_output_plugin() {
        let output_result = serde_json::json!({});
        let route = vec![
            EdgeTraversal {
                edge_id: EdgeId(0),
                access_cost: Cost::from(1.0),
                traversal_cost: Cost::from(1.0),
                result_state: vec![StateVar(0.0)],
            },
            EdgeTraversal {
                edge_id: EdgeId(1),
                access_cost: Cost::from(1.0),
                traversal_cost: Cost::from(1.0),
                result_state: vec![StateVar(0.0)],
            },
            EdgeTraversal {
                edge_id: EdgeId(2),
                access_cost: Cost::from(1.0),
                traversal_cost: Cost::from(1.0),
                result_state: vec![StateVar(0.0)],
            },
        ];
        let EdgeList_plugin = EdgeListOutputPlugin {};
        let updated_output = EdgeList_plugin
            .proccess(&output_result, Ok(&route))
            .unwrap();
        let cost: f64 = updated_output.get_cost().unwrap().into();
        assert_eq!(cost, 6.0);
    }
}
