use super::CombinedTraversalModel;
use crate::model::traversal::{
    default::combined::combined_ops::topological_dependency_sort, TraversalModel,
    TraversalModelError, TraversalModelService,
};
use itertools::Itertools;
use std::sync::Arc;

pub struct CombinedTraversalService {
    services: Vec<Arc<dyn TraversalModelService>>,
}

impl CombinedTraversalService {
    pub fn new(services: Vec<Arc<dyn TraversalModelService>>) -> CombinedTraversalService {
        CombinedTraversalService { services }
    }
}

impl TraversalModelService for CombinedTraversalService {
    fn build(
        &self,
        query: &serde_json::Value,
    ) -> Result<Arc<dyn TraversalModel>, TraversalModelError> {
        let models: Vec<Arc<dyn TraversalModel>> = self
            .services
            .iter()
            .map(|s| {
                let service = s.clone();
                service.build(query)
            })
            .try_collect()?;
        let sorted_models = topological_dependency_sort(&models)?;
        Ok(Arc::new(CombinedTraversalModel::new(sorted_models)))
    }
}
