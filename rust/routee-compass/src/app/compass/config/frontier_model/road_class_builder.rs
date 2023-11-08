use crate::app::compass::config::{
    builders::FrontierModelBuilder, compass_configuration_error::CompassConfigurationError,
    compass_configuration_field::CompassConfigurationField,
    config_json_extension::ConfigJsonExtensions,
};
use routee_compass_core::{
    model::frontier::{default::road_class::RoadClassFrontierModel, frontier_model::FrontierModel},
    util::fs::{read_decoders, read_utils},
};
use std::collections::HashSet;

pub struct RoadClassBuilder {}

impl FrontierModelBuilder for RoadClassBuilder {
    fn build(
        &self,
        parameters: &serde_json::Value,
    ) -> Result<Box<dyn FrontierModel>, CompassConfigurationError> {
        let frontier_key = CompassConfigurationField::Frontier.to_string();
        let road_class_file_key = String::from("road_class_input_file");
        let valid_road_class_key = String::from("valid_road_classes");

        let road_class_file =
            parameters.get_config_path(road_class_file_key.clone(), frontier_key.clone())?;

        let road_classes_json = parameters
            .get(valid_road_class_key.clone())
            .ok_or(CompassConfigurationError::ExpectedFieldForComponent(
                valid_road_class_key.clone(),
                frontier_key.clone(),
            ))?
            .as_array()
            .ok_or(CompassConfigurationError::ExpectedFieldWithType(
                valid_road_class_key.clone(),
                String::from("Array"),
            ))?;
        let road_classes: HashSet<String> = road_classes_json
            .iter()
            .map(|rc| serde_json::from_value::<String>(rc.clone()))
            .collect::<Result<HashSet<String>, _>>()
            .map_err(CompassConfigurationError::SerdeDeserializationError)?;

        let road_class_lookup: Vec<bool> =
            read_utils::read_raw_file(&road_class_file, read_decoders::string, None)?
                .iter()
                .map(|rc| road_classes.contains(rc))
                .collect();

        let m: Box<dyn FrontierModel> = Box::new(RoadClassFrontierModel { road_class_lookup });
        return Ok(m);
    }
}
