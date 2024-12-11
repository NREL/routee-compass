use crate::{
    app::search::search_app::SearchApp,
    plugin::input::{input_plugin::InputPlugin, InputPluginError},
};
use log;
use std::sync::Arc;

pub struct DebugInputPlugin {}

impl InputPlugin for DebugInputPlugin {
    fn process(
        &self,
        input: &mut serde_json::Value,
        _search_app: Arc<SearchApp>,
    ) -> Result<(), InputPluginError> {
        let string = serde_json::to_string_pretty(input)
            .map_err(|e| InputPluginError::JsonError { source: e })?;
        log::debug!("{}", string);
        Ok(())
    }
}
