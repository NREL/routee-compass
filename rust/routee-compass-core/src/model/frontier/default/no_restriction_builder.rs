use crate::model::frontier::{
    default::no_restriction::NoRestriction, FrontierModelBuilder, FrontierModelError,
    FrontierModelService,
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
