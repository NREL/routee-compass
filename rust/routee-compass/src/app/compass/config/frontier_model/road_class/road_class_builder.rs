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
use std::{collections::HashSet, sync::Arc};

use super::road_class_service::RoadClassFrontierService;

pub struct RoadClassBuilder {}

impl FrontierModelBuilder for RoadClassBuilder {
    fn build(
        &self,
        parameters: &serde_json::Value,
    ) -> Result<Arc<dyn FrontierModelService>, FrontierModelError> {
        let frontier_key = CompassConfigurationField::Frontier.to_string();
        let road_class_file_key = String::from("road_class_input_file");
        // let valid_road_class_key = String::from("valid_road_classes");

        let road_class_file = parameters
            .get_config_path(road_class_file_key.clone(), frontier_key.clone())
            .map_err(|e| {
                FrontierModelError::BuildError(format!(
                    "configuration error due to {}: {}",
                    road_class_file_key.clone(),
                    e
                ))
            })?;

        // let road_classes_vec = parameters
        //     .get_config_serde::<Vec<String>>(valid_road_class_key.clone(), frontier_key.clone())
        //     .map_err(|e| {
        //         FrontierModelError::BuildError(format!(
        //             "configuration error due to {}: {}",
        //             valid_road_class_key.clone(),
        //             e
        //         ))
        //     })?;
        // let road_classes: HashSet<String> = HashSet::from_iter(road_classes_vec.to_vec());

        // log::debug!("valid road classes (raw/hashset): {:?}", road_classes_vec);

        let road_class_lookup: Box<[String]> =
            read_utils::read_raw_file(road_class_file.clone(), read_decoders::string, None)
                .map_err(|e| {
                    FrontierModelError::BuildError(format!(
                        "failed to load file at {:?}: {}",
                        road_class_file.clone().to_str(),
                        e
                    ))
                })?;
        // .iter()
        // .map(|rc| road_classes.contains(rc.trim()))
        // .collect();

        // let n_good = road_class_lookup.iter().filter(|r| **r).count();
        // log::debug!(
        //     "{}/{} links have a valid road class",
        //     n_good,
        //     road_class_lookup.len()
        // );

        let m: Arc<dyn FrontierModelService> = Arc::new(RoadClassFrontierService {
            road_class_lookup: Arc::new(road_class_lookup),
        });
        Ok(m)
    }
}
