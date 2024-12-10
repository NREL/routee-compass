use crate::plugin::input::{input_plugin::InputPlugin, InputPluginError};

pub struct InjectInputPlugin {
    key: String,
    value: serde_json::Value,
    overwrite: bool,
}

impl InjectInputPlugin {
    pub fn new(
        key: String,
        value: serde_json::Value,
        overwrite: Option<bool>,
    ) -> InjectInputPlugin {
        InjectInputPlugin {
            key,
            value,
            overwrite: overwrite.unwrap_or(true),
        }
    }
}

impl InputPlugin for InjectInputPlugin {
    fn process(&self, input: &mut serde_json::Value) -> Result<(), InputPluginError> {
        if !self.overwrite {
            if let Some(obj) = input.as_object() {
                if obj.contains_key(&self.key) {
                    return Err(InputPluginError::InputPluginFailed(format!("inject plugin for key {} has a no-overwrite policy but encountered query with existing value", self.key)));
                }
            } else {
                return Err(InputPluginError::UnexpectedQueryStructure(String::from(
                    "query is not a JSON object",
                )));
            }
        }
        input[self.key.clone()] = self.value.clone();
        Ok(())
    }
}
