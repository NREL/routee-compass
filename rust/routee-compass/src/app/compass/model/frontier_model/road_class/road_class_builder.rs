use super::{road_class_parser::RoadClassParser, road_class_service::RoadClassFrontierService};
use crate::app::compass::{CompassConfigurationField, ConfigJsonExtensions};
use kdam::Bar;
use routee_compass_core::{
    model::frontier::{FrontierModelBuilder, FrontierModelError, FrontierModelService},
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

        let road_class_lookup: Box<[u8]> = read_utils::read_raw_file(
            &road_class_file,
            read_decoders::u8,
            Some(Bar::builder().desc("road class")),
            None,
        )
        .map_err(|e| {
            FrontierModelError::BuildError(format!(
                "failed to load file at {:?}: {}",
                road_class_file.clone().to_str(),
                e
            ))
        })?;

        let road_class_parser = parameters
            .get_config_serde_optional::<RoadClassParser>(
                &"road_class_parser",
                &"RoadClassFrontierModel",
            )
            .map_err(|e| {
                FrontierModelError::BuildError(format!(
                    "unable to deserialize road_class_parser: {}",
                    e
                ))
            })?
            .unwrap_or_default();

        let m: Arc<dyn FrontierModelService> = Arc::new(RoadClassFrontierService {
            road_class_lookup: Arc::new(road_class_lookup),
            road_class_parser,
        });
        Ok(m)
    }
}
