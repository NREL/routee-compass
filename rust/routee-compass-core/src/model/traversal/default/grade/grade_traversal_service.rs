use super::{GradeTraversalEngine, GradeTraversalModel};
use crate::model::traversal::{TraversalModel, TraversalModelError, TraversalModelService};
use std::sync::Arc;

pub struct GradeTraversalService {
    engine: Arc<GradeTraversalEngine>,
}

impl GradeTraversalService {
    pub fn new(engine: Arc<GradeTraversalEngine>) -> GradeTraversalService {
        GradeTraversalService { engine }
    }
}

impl TraversalModelService for GradeTraversalService {
    fn build(
        &self,
        _query: &serde_json::Value,
    ) -> Result<Arc<dyn TraversalModel>, TraversalModelError> {
        let model = GradeTraversalModel::new(self.engine.clone());
        Ok(Arc::new(model))
    }
}
