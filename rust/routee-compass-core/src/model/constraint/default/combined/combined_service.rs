use super::combined_model::CombinedConstraintModel;
use crate::model::{
    constraint::{ConstraintModel, ConstraintModelError, ConstraintModelService},
    state::StateModel,
};
use std::sync::Arc;

#[derive(Clone)]
pub struct CombinedFrontierService {
    pub inner_services: Vec<Arc<dyn ConstraintModelService>>,
}

impl ConstraintModelService for CombinedFrontierService {
    fn build(
        &self,
        query: &serde_json::Value,
        state_model: Arc<StateModel>,
    ) -> Result<Arc<dyn ConstraintModel>, ConstraintModelError> {
        let inner_models = self
            .inner_services
            .iter()
            .map(|s| s.build(query, state_model.clone()))
            .collect::<Result<Vec<Arc<dyn ConstraintModel>>, ConstraintModelError>>()?;
        let model = CombinedConstraintModel { inner_models };
        Ok(Arc::new(model))
    }
}
