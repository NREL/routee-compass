use crate::{
    app::compass::config::{
        builders::OutputPluginBuilder, compass_configuration_error::CompassConfigurationError,
    },
    plugin::output::output_plugin::OutputPlugin,
};

use super::plugin::EdgeListOutputPlugin;

pub struct EdgeListOutputPluginBuilder {}

impl OutputPluginBuilder for EdgeListOutputPluginBuilder {
    fn build(
        &self,
        parameters: &serde_json::Value,
    ) -> Result<Box<dyn OutputPlugin>, CompassConfigurationError> {
        Ok(Box::new(EdgeListOutputPlugin {}))
    }
}
