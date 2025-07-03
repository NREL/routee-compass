use crate::config::ConfigJsonExtensions;
use crate::{
    model::access::{
        default::turn_delays::{
            EdgeHeading, TurnDelayAccessModelEngine, TurnDelayAccessModelService,
            TurnDelayModelConfig,
        },
        AccessModelBuilder, AccessModelError, AccessModelService,
    },
    util::fs::read_utils,
};
use kdam::Bar;
use std::sync::Arc;

pub struct TurnDelayAccessModelBuilder {}

impl AccessModelBuilder for TurnDelayAccessModelBuilder {
    fn build(
        &self,
        parameters: &serde_json::Value,
    ) -> Result<Arc<dyn AccessModelService>, AccessModelError> {
        let file_path = parameters
            .get_config_path(&"edge_heading_input_file", &"turn delay access model")
            .map_err(|e| {
                AccessModelError::BuildError(format!(
                    "failure reading 'edge_heading_input_file' from access model configuration: {}",
                    e
                ))
            })?;
        let edge_headings = read_utils::from_csv::<EdgeHeading>(
            &file_path.as_path(),
            true,
            Some(Bar::builder().desc("edge headings")),
            None,
        )
        .map_err(|e| {
            AccessModelError::BuildError(format!(
                "error reading headings from file {:?}: {}",
                file_path, e
            ))
        })?;
        let turn_delay_model_config = parameters
            .get_config_serde::<TurnDelayModelConfig>(
                &"turn_delay_model",
                &"turn delay access model",
            )
            .map_err(|e| {
                AccessModelError::BuildError(format!(
                    "failure reading 'turn_delay_model' from access model configuration: {}",
                    e
                ))
            })?;

        let engine = TurnDelayAccessModelEngine {
            edge_headings,
            turn_delay_model: turn_delay_model_config.into(),
        };
        let service = TurnDelayAccessModelService {
            engine: Arc::new(engine),
        };
        Ok(Arc::new(service))
    }
}
