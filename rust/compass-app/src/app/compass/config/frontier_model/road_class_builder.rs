use std::collections::HashSet;

use compass_core::model::{
    frontier::{default::road_class::RoadClassFrontierModel, frontier_model::FrontierModel},
    property::road_class::RoadClass,
};

use crate::app::compass::config::{
    builders::FrontierModelBuilder, compass_configuration_error::CompassConfigurationError,
};

pub struct RoadClassBuilder {}

impl FrontierModelBuilder for RoadClassBuilder {
    fn build(
        &self,
        parameters: &serde_json::Value,
    ) -> Result<Box<dyn FrontierModel>, CompassConfigurationError> {
        let valid_road_class_key = String::from("valid_road_classes");
        let road_classes_json = parameters
            .get(&valid_road_class_key)
            .ok_or(CompassConfigurationError::ExpectedFieldForComponent(
                valid_road_class_key.clone(),
                String::from("RoadClassFrontierModel"),
            ))?
            .as_array()
            .ok_or(CompassConfigurationError::ExpectedFieldWithType(
                valid_road_class_key.clone(),
                String::from("Array"),
            ))?;
        let road_classes: HashSet<RoadClass> = road_classes_json
            .iter()
            .map(|rc| serde_json::from_value::<RoadClass>(rc.clone()))
            .collect::<Result<HashSet<RoadClass>, _>>()
            .map_err(CompassConfigurationError::SerdeDeserializationError)?;

        let m: Box<dyn FrontierModel> = Box::new(RoadClassFrontierModel {
            valid_road_classes: road_classes,
        });
        return Ok(m);
    }
}
