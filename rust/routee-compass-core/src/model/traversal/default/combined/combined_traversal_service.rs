use super::CombinedTraversalModel;
use crate::model::{
    state::{InputFeature, StateVariableConfig},
    traversal::{
        default::combined::combined_ops::topological_dependency_sort_services, TraversalModel,
        TraversalModelError, TraversalModelService,
    },
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
    fn input_features(&self) -> Vec<InputFeature> {
        self.services
            .iter()
            .flat_map(|s| s.input_features())
            .collect()
    }

    fn output_features(&self) -> Vec<(String, StateVariableConfig)> {
        self.services
            .iter()
            .flat_map(|s| s.output_features())
            .collect()
    }

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
        let sorted_models = topological_dependency_sort_services(&self.services, &models)?;
        Ok(Arc::new(CombinedTraversalModel::new(sorted_models)))
    }
}
