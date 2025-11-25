use crate::model::filter::{
    default::no_restriction::NoRestriction, FilterModelBuilder, FilterModelError,
    FilterModelService,
};
use std::sync::Arc;

pub struct NoRestrictionBuilder {}

impl FilterModelBuilder for NoRestrictionBuilder {
    fn build(
        &self,
        _parameters: &serde_json::Value,
    ) -> Result<Arc<dyn FilterModelService>, FilterModelError> {
        Ok(Arc::new(NoRestriction {}))
    }
}
