use std::sync::Arc;

use crate::{
    app::compass::model::{builders::OutputPluginBuilder, CompassConfigurationError},
    plugin::output::output_plugin::OutputPlugin,
};

use super::plugin::SummaryOutputPlugin;

pub struct SummaryOutputPluginBuilder {}

impl OutputPluginBuilder for SummaryOutputPluginBuilder {
    fn build(
        &self,
        _parameters: &serde_json::Value,
    ) -> Result<Arc<dyn OutputPlugin>, CompassConfigurationError> {
        Ok(Arc::new(SummaryOutputPlugin {}))
    }
}
