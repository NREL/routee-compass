use crate::app::compass::config::{
    builders::{TraversalModelBuilder, TraversalModelService},
    compass_configuration_error::CompassConfigurationError,
};
use compass_core::model::traversal::default::distance::DistanceModel;
use compass_core::model::traversal::traversal_model::TraversalModel;
use std::sync::Arc;

pub struct DistanceBuilder {}

pub struct DistanceService {}

impl TraversalModelBuilder for DistanceBuilder {
    fn build(
        &self,
        _parameters: &serde_json::Value,
    ) -> Result<Arc<dyn TraversalModelService>, CompassConfigurationError> {
        let m: Arc<dyn TraversalModelService> = Arc::new(DistanceService {});
        return Ok(m);
    }
}

impl TraversalModelService for DistanceService {
    fn build(
        &self,
        parameters: &serde_json::Value,
    ) -> Result<Arc<dyn TraversalModel>, CompassConfigurationError> {
        let m: Arc<dyn TraversalModel> = Arc::new(DistanceModel {});
        return Ok(m);
    }
}
