use routee_compass_core::model::frontier::{
    default::no_restriction::NoRestriction, frontier_model_builder::FrontierModelBuilder,
    frontier_model_error::FrontierModelError, frontier_model_service::FrontierModelService,
};
use std::sync::Arc;

pub struct NoRestrictionBuilder {}

impl FrontierModelBuilder for NoRestrictionBuilder {
    fn build(
        &self,
        _parameters: &serde_json::Value,
    ) -> Result<Arc<dyn FrontierModelService>, FrontierModelError> {
        Ok(Arc::new(NoRestriction {}))
    }
}
