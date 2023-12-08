use std::sync::Arc;

use crate::model::frontier::{
    frontier_model::FrontierModel, frontier_model_error::FrontierModelError,
    frontier_model_service::FrontierModelService,
};

#[derive(Clone)]
pub struct NoRestriction {}

impl FrontierModel for NoRestriction {}

impl FrontierModelService for NoRestriction {
    fn build(
        &self,
        _query: &serde_json::Value,
    ) -> Result<Arc<dyn FrontierModel>, FrontierModelError> {
        Ok(Arc::new(self.clone()))
    }
}
