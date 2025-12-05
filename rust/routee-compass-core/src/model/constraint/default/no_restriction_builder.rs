use crate::model::constraint::{
    default::no_restriction::NoRestriction, ConstraintModelBuilder, ConstraintModelError,
    ConstraintModelService,
};
use std::sync::Arc;

pub struct NoRestrictionBuilder {}

impl ConstraintModelBuilder for NoRestrictionBuilder {
    fn build(
        &self,
        _parameters: &serde_json::Value,
    ) -> Result<Arc<dyn ConstraintModelService>, ConstraintModelError> {
        Ok(Arc::new(NoRestriction {}))
    }
}
