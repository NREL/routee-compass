use super::plugin::SummaryOutputPlugin;
use crate::{
    app::compass::CompassComponentError,
    plugin::output::{OutputPlugin, OutputPluginBuilder},
};
use std::sync::Arc;

pub struct SummaryOutputPluginBuilder {}

impl OutputPluginBuilder for SummaryOutputPluginBuilder {
    fn build(
        &self,
        _parameters: &serde_json::Value,
    ) -> Result<Arc<dyn OutputPlugin>, CompassComponentError> {
        Ok(Arc::new(SummaryOutputPlugin {}))
    }
}
