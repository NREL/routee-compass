use super::plugin::SummaryOutputPlugin;
use crate::{
    app::compass::CompassComponentError,
    plugin::{
        output::{default::summary::SummaryConfig, OutputPlugin, OutputPluginBuilder},
        PluginError,
    },
};
use std::sync::Arc;

pub struct SummaryOutputPluginBuilder {}

impl OutputPluginBuilder for SummaryOutputPluginBuilder {
    fn build(
        &self,
        parameters: &serde_json::Value,
    ) -> Result<Arc<dyn OutputPlugin>, CompassComponentError> {
        let conf: SummaryConfig = serde_json::from_value(parameters.clone()).map_err(|e| {
            PluginError::BuildFailed(format!("failure reading summary output plugin config: {e}"))
        })?;
        Ok(Arc::new(SummaryOutputPlugin::new(conf)))
    }
}
