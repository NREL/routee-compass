use compass_core::model::traversal::default_models::distance::DistanceModel;
use compass_core::model::traversal::traversal_model::TraversalModel;

use crate::app::compass::conf_v2::{
    compass_configuration_error::CompassConfigurationError,
    traversal_model_builder::TraversalModelBuilder,
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
