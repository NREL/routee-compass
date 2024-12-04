use crate::plugin::input::{input_plugin::InputPlugin, InputPluginError};

pub struct DebugInputPlugin {}

impl InputPlugin for DebugInputPlugin {
    fn process(&self, input: &mut serde_json::Value) -> Result<(), InputPluginError> {
        let string = serde_json::to_string_pretty(input)
            .map_err(|e| InputPluginError::JsonError { source: e })?;
        println!("{}", string);
        Ok(())
    }
}
