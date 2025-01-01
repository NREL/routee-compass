use super::plugin::SummaryOutputPlugin;
use crate::{
    app::compass::CompassConfigurationError,
    plugin::output::{OutputPlugin, OutputPluginBuilder},
};
use std::sync::Arc;

pub struct SummaryOutputPluginBuilder {}

impl OutputPluginBuilder for SummaryOutputPluginBuilder {
    fn build(
        &self,
        _parameters: &serde_json::Value,
    ) -> Result<Arc<dyn OutputPlugin>, CompassConfigurationError> {
        Ok(Arc::new(SummaryOutputPlugin {}))
    }
}
