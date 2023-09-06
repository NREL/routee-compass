use crate::plugin::input::input_json_extensions::InputJsonExtensions;
use crate::plugin::input::input_plugin::InputPlugin;
use crate::plugin::plugin_error::PluginError;
use compass_core::{
    map::rtree::VertexRTree, model::property::vertex::Vertex, util::fs::read_utils,
};

/// Builds an input plugin that duplicates queries if array-valued fields are present
/// by stepping through each combination of value
pub struct GridSearchPlugin {}

impl InputPlugin for GridSearchPlugin {
    fn process(&self, input: &serde_json::Value) -> Result<Vec<serde_json::Value>, PluginError> {
        let map = input
            .as_object()
            .ok_or(PluginError::UnexpectedQueryStructure(format!(
                "{:?}",
                input
            )))?;
        let y = map.into_iter().map(|(k, v)| match v {
            serde_json::Value::Array(values) => {}
            _ => {}
        });

        // https://docs.rs/itertools/latest/itertools/structs/struct.Permutations.html

        Ok(vec![])
    }
}
