use crate::app::compass::config::config_json_extension::ConfigJsonExtensions;
use routee_compass_core::{
    model::access::{
        access_model_builder::AccessModelBuilder,
        access_model_error::AccessModelError,
        access_model_service::AccessModelService,
        default::turn_delays::{
            edge_heading::EdgeHeading, turn_delay_access_model_engine::TurnDelayAccessModelEngine,
            turn_delay_access_model_service::TurnDelayAccessModelService,
            turn_delay_model::TurnDelayModel,
        },
    },
    util::fs::read_utils,
};
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
        let edge_headings = read_utils::from_csv::<EdgeHeading>(&file_path.as_path(), true, None)
            .map_err(|e| {
            AccessModelError::BuildError(format!(
                "error reading headings from file {:?}: {}",
                file_path, e
            ))
        })?;
        let turn_delay_model = parameters
            .get_config_serde::<TurnDelayModel>(&"turn_delay_model", &"turn delay access model")
            .map_err(|e| {
                AccessModelError::BuildError(format!(
                    "failure reading 'turn_delay_model' from access model configuration: {}",
                    e
                ))
            })?;
        let time_feature_name = parameters
            .get_config_serde_optional::<String>(&"time_feature_name", &"turn delay access model")
            .map_err(|e| {
                AccessModelError::BuildError(format!(
                    "failure reading 'time_unit' from access model configuration: {}",
                    e
                ))
            })?
            .unwrap_or_else(|| String::from("time"));
        let engine = TurnDelayAccessModelEngine {
            edge_headings,
            turn_delay_model,
            time_feature_name,
        };
        let service = TurnDelayAccessModelService {
            engine: Arc::new(engine),
        };
        Ok(Arc::new(service))
    }
}
