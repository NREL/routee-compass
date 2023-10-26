use routee_compass_core::model::frontier::{
    default::no_restriction::NoRestriction, frontier_model::FrontierModel,
};

use crate::app::compass::config::{
    builders::FrontierModelBuilder, compass_configuration_error::CompassConfigurationError,
};

pub struct NoRestrictionBuilder {}

impl FrontierModelBuilder for NoRestrictionBuilder {
    fn build(
        &self,
        _parameters: &serde_json::Value,
    ) -> Result<Box<dyn FrontierModel>, CompassConfigurationError> {
        Ok(Box::new(NoRestriction {}))
    }
}
