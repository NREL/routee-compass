use compass_core::model::traversal::default::distance::DistanceModel;
use compass_core::model::traversal::traversal_model::TraversalModel;

use crate::app::compass::config::{
    compass_configuration_error::CompassConfigurationError,
    builders::TraversalModelBuilder,
};

pub struct DistanceBuilder {}

impl TraversalModelBuilder for DistanceBuilder {
    fn build(
        &self,
        _parameters: &serde_json::Value,
    ) -> Result<Box<dyn TraversalModel>, CompassConfigurationError> {
        let m: Box<dyn TraversalModel> = Box::new(DistanceModel {});
        return Ok(m);
    }
}
