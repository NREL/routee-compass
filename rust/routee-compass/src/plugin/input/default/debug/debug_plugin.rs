use crate::plugin::{input::input_plugin::InputPlugin, plugin_error::PluginError};

pub struct DebugInputPlugin {}

impl InputPlugin for DebugInputPlugin {
    fn process(&self, input: &serde_json::Value) -> Result<Vec<serde_json::Value>, PluginError> {
        let string = serde_json::to_string_pretty(input).map_err(PluginError::JsonError)?;
        println!("{}", string);
        Ok(vec![input.clone()])
    }
}
