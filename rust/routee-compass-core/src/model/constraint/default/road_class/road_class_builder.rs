use super::road_class_service::RoadClassFrontierService;
use crate::config::{CompassConfigurationField, ConfigJsonExtensions};
use crate::{
    model::constraint::{ConstraintModelBuilder, ConstraintModelError, ConstraintModelService},
    util::fs::{read_decoders, read_utils},
};
use kdam::Bar;
use std::sync::Arc;

pub struct RoadClassBuilder {}

impl ConstraintModelBuilder for RoadClassBuilder {
    fn build(
        &self,
        parameters: &serde_json::Value,
    ) -> Result<Arc<dyn ConstraintModelService>, ConstraintModelError> {
        let constraint_key = CompassConfigurationField::Constraint.to_string();
        let road_class_file_key = String::from("road_class_input_file");

        let road_class_file = parameters
            .get_config_path(&road_class_file_key, &constraint_key)
            .map_err(|e| {
                ConstraintModelError::BuildError(format!(
                    "configuration error due to {}: {}",
                    road_class_file_key.clone(),
                    e
                ))
            })?;

        let road_class_lookup: Box<[String]> = read_utils::read_raw_file(
            &road_class_file,
            read_decoders::string,
            Some(Bar::builder().desc("road class")),
            None,
        )
        .map_err(|e| {
            ConstraintModelError::BuildError(format!(
                "failed to load file at {:?}: {}",
                road_class_file.clone().to_str(),
                e
            ))
        })?;

        let m: Arc<dyn ConstraintModelService> = Arc::new(RoadClassFrontierService {
            road_class_by_edge: Arc::new(road_class_lookup),
            // road_class_parser,
        });
        Ok(m)
    }
}
