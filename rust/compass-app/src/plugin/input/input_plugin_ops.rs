use crate::plugin::plugin_error::PluginError;

use super::InputPlugin;

pub fn apply_input_plugins(
    input_queries: Vec<serde_json::Value>,
    plugins: &Vec<InputPlugin>,
) -> Result<Vec<serde_json::Value>, PluginError> {
    input_queries
        .iter()
        .map(|query| {
            let init_acc: Result<serde_json::Value, PluginError> = Ok(query.clone());
            plugins.iter().fold(init_acc, move |acc, plugin| match acc {
                Err(e) => Err(e),
                Ok(json) => plugin(&json),
            })
        })
        .collect()
}
