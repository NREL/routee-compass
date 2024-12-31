use super::combined_model::CombinedFrontierModel;
use routee_compass_core::model::{
    frontier::{
        frontier_model::FrontierModel, frontier_model_error::FrontierModelError,
        frontier_model_service::FrontierModelService,
    },
    state::StateModel,
};
use std::sync::Arc;

#[derive(Clone)]
pub struct CombinedFrontierService {
    pub inner_services: Vec<Arc<dyn FrontierModelService>>,
}

impl FrontierModelService for CombinedFrontierService {
    fn build(
        &self,
        query: &serde_json::Value,
        state_model: Arc<StateModel>,
    ) -> Result<Arc<dyn FrontierModel>, FrontierModelError> {
        let inner_models = self
            .inner_services
            .iter()
            .map(|s| s.build(query, state_model.clone()))
            .collect::<Result<Vec<Arc<dyn FrontierModel>>, FrontierModelError>>()?;
        let model = CombinedFrontierModel { inner_models };
        Ok(Arc::new(model))
    }
}
