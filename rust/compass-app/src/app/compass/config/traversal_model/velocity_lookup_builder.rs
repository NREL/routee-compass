use compass_core::model::traversal::traversal_model::TraversalModel;
use compass_core::model::{
    traversal::default::velocity_lookup::VelocityLookupModel, units::TimeUnit,
};

use crate::app::compass::compass_configuration_field::CompassConfigurationField;
use crate::app::compass::config::{
    compass_configuration_error::CompassConfigurationError,
    traversal_model_builder::TraversalModelBuilder,
};

pub struct VelocityLookupBuilder {}

impl TraversalModelBuilder for VelocityLookupBuilder {
    fn build(
        &self,
        parameters: &serde_json::Value,
    ) -> Result<Box<dyn TraversalModel>, CompassConfigurationError> {
        let filename_key = String::from("filename");
        let time_unit_key = String::from("time_unit");
        let traversal_key = CompassConfigurationField::Traversal.to_string();
        let filename = parameters
            .get(&filename_key)
            .ok_or(CompassConfigurationError::ExpectedFieldForComponent(
                filename_key.clone(),
                traversal_key.clone(),
            ))?
            .as_str()
            .map(String::from)
            .ok_or(CompassConfigurationError::ExpectedFieldWithType(
                filename_key.clone(),
                String::from("String"),
            ))?;

        let time_unit = parameters
            .get(&time_unit_key)
            .map(|t| serde_json::from_value::<TimeUnit>(t.clone()))
            .ok_or(CompassConfigurationError::ExpectedFieldForComponent(
                filename_key.clone(),
                time_unit_key.clone(),
            ))?
            .map_err(CompassConfigurationError::SerdeDeserializationError)?;

        let m = VelocityLookupModel::from_file(&filename, time_unit)
            .map_err(CompassConfigurationError::TraversalModelError)?;
        return Ok(Box::new(m));
    }
}
