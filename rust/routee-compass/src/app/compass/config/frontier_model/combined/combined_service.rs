use routee_compass_core::model::frontier::{
    frontier_model::FrontierModel, frontier_model_error::FrontierModelError,
    frontier_model_service::FrontierModelService,
};
use std::sync::Arc;

use super::combined_model::CombinedFrontierModel;

#[derive(Clone)]
pub struct CombinedFrontierService {
    pub inner_services: Vec<Arc<dyn FrontierModelService>>,
}

impl FrontierModelService for CombinedFrontierService {
    fn build(
        &self,
        query: &serde_json::Value,
    ) -> Result<Arc<dyn FrontierModel>, FrontierModelError> {
        let inner_models = self
            .inner_services
            .iter()
            .map(|s| s.build(query))
            .collect::<Result<Vec<Arc<dyn FrontierModel>>, FrontierModelError>>()?;
        let model = CombinedFrontierModel { inner_models };
        Ok(Arc::new(model))
    }
}
