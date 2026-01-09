use super::{CustomTraversalEngine, CustomTraversalModel};
use crate::model::{
    state::{InputFeature, StateModel, StateVariableConfig},
    traversal::{
        traversal_model::TraversalModel, traversal_model_error::TraversalModelError,
        traversal_model_service::TraversalModelService,
    },
};
use std::sync::Arc;

pub struct CustomTraversalService {
    pub engine: Arc<CustomTraversalEngine>,
}

impl TraversalModelService for CustomTraversalService {
    fn input_features(&self) -> Vec<InputFeature> {
        vec![]
    }

    fn output_features(&self) -> Vec<(String, StateVariableConfig)> {
        let feature = self.engine.output_feature();
        let name = self.engine.config().custom_type.clone();
        vec![(name, feature)]
    }

    fn build(
        &self,
        _parameters: &serde_json::Value,
        _state_model: Arc<StateModel>,
    ) -> Result<Arc<dyn TraversalModel>, TraversalModelError> {
        let model = CustomTraversalModel::new(self.engine.clone());
        Ok(Arc::new(model))
    }
}
