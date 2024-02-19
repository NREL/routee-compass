use crate::plugin::{input::input_plugin::InputPlugin, plugin_error::PluginError};

pub struct DebugInputPlugin {}

impl InputPlugin for DebugInputPlugin {
    fn process(&self, input: &mut serde_json::Value) -> Result<(), PluginError> {
        let string = serde_json::to_string_pretty(input).map_err(PluginError::JsonError)?;
        println!("{}", string);
        Ok(())
    }
}
