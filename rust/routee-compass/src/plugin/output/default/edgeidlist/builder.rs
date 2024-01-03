use std::sync::Arc;

use crate::{
    app::compass::config::{
        builders::OutputPluginBuilder, compass_configuration_error::CompassConfigurationError,
    },
    plugin::output::output_plugin::OutputPlugin,
};

use super::plugin::EdgeIdListOutputPlugin;

pub struct EdgeIdListOutputPluginBuilder {}

impl OutputPluginBuilder for EdgeIdListOutputPluginBuilder {
    fn build(
        &self,
        _parameters: &serde_json::Value,
    ) -> Result<Arc<dyn OutputPlugin>, CompassConfigurationError> {
        Ok(Arc::new(EdgeIdListOutputPlugin {}))
    }
}
