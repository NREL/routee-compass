use std::sync::Arc;

use routee_compass_core::{
    config::ConfigJsonExtensions,
    model::label::{
        label_model_builder::LabelModelBuilder, label_model_error::LabelModelError,
        label_model_service::LabelModelService,
    },
};
use serde::{Deserialize, Serialize};

use crate::model::charging::soc_label_model::SOCLabelModel;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum SOCLabelConfig {
    ExplicitPercentBins(Vec<u64>),
    PercentRange { start: u64, end: u64, step: u64 },
}

pub struct SOCLabelModelBuilder;

impl LabelModelBuilder for SOCLabelModelBuilder {
    fn build(
        &self,
        parameters: &serde_json::Value,
    ) -> Result<Arc<dyn LabelModelService>, LabelModelError> {
        let config: SOCLabelConfig = parameters
            .get_config_serde(&"soc_label_config", &"soc_label")
            .map_err(|e| {
                LabelModelError::LabelModelError(format!(
                    "Failed to deserialize SOCLabelConfig: {}",
                    e
                ))
            })?;

        let model = match config {
            SOCLabelConfig::ExplicitPercentBins(soc_percent_bins) => {
                SOCLabelModel::new(soc_percent_bins)
            }
            SOCLabelConfig::PercentRange { start, end, step } => {
                SOCLabelModel::from_range(start, end, step)
            }
        };
        Ok(Arc::new(model))
    }
}
