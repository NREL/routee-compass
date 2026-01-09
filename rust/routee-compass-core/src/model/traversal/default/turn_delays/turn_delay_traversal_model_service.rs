use super::TurnDelayTraversalModel;
use super::TurnDelayTraversalModelEngine;
use crate::model::traversal::TraversalModel;
use crate::model::traversal::TraversalModelError;
use crate::model::traversal::TraversalModelService;
use std::sync::Arc;

pub struct TurnDelayTraversalModelService {
    pub engine: Arc<TurnDelayTraversalModelEngine>,
    pub include_trip_time: bool,
}

impl TurnDelayTraversalModelService {}

impl TraversalModelService for TurnDelayTraversalModelService {
    fn build(
        &self,
        _query: &serde_json::Value,
    ) -> Result<Arc<dyn TraversalModel>, TraversalModelError> {
        let model = TurnDelayTraversalModel::new(self.engine.clone(), self.include_trip_time);
        Ok(Arc::new(model))
    }
}
