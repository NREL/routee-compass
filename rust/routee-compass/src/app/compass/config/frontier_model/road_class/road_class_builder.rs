use super::road_class_service::RoadClassFrontierService;
use crate::app::compass::config::{
    compass_configuration_field::CompassConfigurationField,
    config_json_extension::ConfigJsonExtensions,
};
use routee_compass_core::{
    model::frontier::{
        frontier_model_builder::FrontierModelBuilder, frontier_model_error::FrontierModelError,
        frontier_model_service::FrontierModelService,
    },
    util::fs::{read_decoders, read_utils},
};
use std::sync::Arc;

pub struct RoadClassBuilder {}

impl FrontierModelBuilder for RoadClassBuilder {
    fn build(
        &self,
        parameters: &serde_json::Value,
    ) -> Result<Arc<dyn FrontierModelService>, FrontierModelError> {
        let frontier_key = CompassConfigurationField::Frontier.to_string();
        let road_class_file_key = String::from("road_class_input_file");

        let road_class_file = parameters
            .get_config_path(&road_class_file_key, &frontier_key)
            .map_err(|e| {
                FrontierModelError::BuildError(format!(
                    "configuration error due to {}: {}",
                    road_class_file_key.clone(),
                    e
                ))
            })?;

        let road_class_lookup: Box<[String]> =
            read_utils::read_raw_file(&road_class_file, read_decoders::string, None).map_err(
                |e| {
                    FrontierModelError::BuildError(format!(
                        "failed to load file at {:?}: {}",
                        road_class_file.clone().to_str(),
                        e
                    ))
                },
            )?;

        let m: Arc<dyn FrontierModelService> = Arc::new(RoadClassFrontierService {
            road_class_lookup: Arc::new(road_class_lookup),
        });
        Ok(m)
    }
}
