use super::TurnDelayTraversalModel;
use super::TurnDelayTraversalModelEngine;
use crate::model::traversal::TraversalModel;
use crate::model::traversal::TraversalModelError;
use crate::model::traversal::TraversalModelService;
use std::sync::Arc;

pub struct TurnDelayTraversalModelService {
    pub engine: Arc<TurnDelayTraversalModelEngine>,
}

impl TurnDelayTraversalModelService {}

impl TraversalModelService for TurnDelayTraversalModelService {
    fn build(
        &self,
        _query: &serde_json::Value,
    ) -> Result<Arc<dyn TraversalModel>, TraversalModelError> {
        let model = TurnDelayTraversalModel {
            engine: self.engine.clone(),
        };
        Ok(Arc::new(model))
    }
}
