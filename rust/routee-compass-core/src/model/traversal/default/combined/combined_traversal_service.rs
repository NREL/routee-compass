use super::CombinedTraversalModel;
use crate::model::traversal::{TraversalModel, TraversalModelError, TraversalModelService};
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
        Ok(Arc::new(CombinedTraversalModel::new(models)))
    }
}
