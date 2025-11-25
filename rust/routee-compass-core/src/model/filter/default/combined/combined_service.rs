use super::combined_model::CombinedFilterModel;
use crate::model::{
    filter::{FilterModel, FilterModelError, FilterModelService},
    state::StateModel,
};
use std::sync::Arc;

#[derive(Clone)]
pub struct CombinedFrontierService {
    pub inner_services: Vec<Arc<dyn FilterModelService>>,
}

impl FilterModelService for CombinedFrontierService {
    fn build(
        &self,
        query: &serde_json::Value,
        state_model: Arc<StateModel>,
    ) -> Result<Arc<dyn FilterModel>, FilterModelError> {
        let inner_models = self
            .inner_services
            .iter()
            .map(|s| s.build(query, state_model.clone()))
            .collect::<Result<Vec<Arc<dyn FilterModel>>, FilterModelError>>()?;
        let model = CombinedFilterModel { inner_models };
        Ok(Arc::new(model))
    }
}
