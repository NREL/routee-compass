use super::{
    EdgeHeading, TurnDelayModelConfig, TurnDelayTraversalModelEngine,
    TurnDelayTraversalModelService,
};
use crate::config::ConfigJsonExtensions;
use crate::{
    model::traversal::{TraversalModelBuilder, TraversalModelError, TraversalModelService},
    util::fs::read_utils,
};
use kdam::Bar;
use std::sync::Arc;

pub struct TurnDelayTraversalModelBuilder {}

impl TraversalModelBuilder for TurnDelayTraversalModelBuilder {
    fn build(
        &self,
        parameters: &serde_json::Value,
    ) -> Result<Arc<dyn TraversalModelService>, TraversalModelError> {
        let file_path = parameters
            .get_config_path(&"edge_heading_input_file", &"turn delay access model")
            .map_err(|e| {
                TraversalModelError::BuildError(format!(
                    "failure reading 'edge_heading_input_file' from access model configuration: {e}"
                ))
            })?;
        let edge_headings = read_utils::from_csv::<EdgeHeading>(
            &file_path.as_path(),
            true,
            Some(Bar::builder().desc("edge headings")),
            None,
        )
        .map_err(|e| {
            TraversalModelError::BuildError(format!(
                "error reading headings from file {file_path:?}: {e}"
            ))
        })?;
        let turn_delay_model_config = parameters
            .get_config_serde::<TurnDelayModelConfig>(
                &"turn_delay_model",
                &"turn delay access model",
            )
            .map_err(|e| {
                TraversalModelError::BuildError(format!(
                    "failure reading 'turn_delay_model' from access model configuration: {e}"
                ))
            })?;

        let engine = TurnDelayTraversalModelEngine {
            edge_headings,
            turn_delay_model: turn_delay_model_config.into(),
        };
        let service = TurnDelayTraversalModelService {
            engine: Arc::new(engine),
        };
        Ok(Arc::new(service))
    }
}
