use crate::model::{
    access::{AccessModel, AccessModelError, AccessModelService},
    network::{Edge, Vertex},
    state::{StateFeature, StateModel, StateVariable},
};
use itertools::Itertools;
use std::sync::Arc;

pub struct CombinedAccessModelService {
    pub services: Vec<Arc<dyn AccessModelService>>,
}

pub struct CombinedAccessModel {
    pub models: Vec<Arc<dyn AccessModel>>,
}

impl AccessModelService for CombinedAccessModelService {
    fn build(&self, query: &serde_json::Value) -> Result<Arc<dyn AccessModel>, AccessModelError> {
        let models = self
            .services
            .iter()
            .map(|m| m.build(query))
            .collect::<Result<_, _>>()?;
        Ok(Arc::new(CombinedAccessModel { models }))
    }
}

impl AccessModel for CombinedAccessModel {
    fn state_features(&self) -> Vec<(String, StateFeature)> {
        self.models
            .iter()
            .flat_map(|m| m.state_features())
            .collect_vec()
    }

    fn access_edge(
        &self,
        traversal: (&Vertex, &Edge, &Vertex, &Edge, &Vertex),
        state: &mut Vec<StateVariable>,
        state_model: &StateModel,
    ) -> Result<(), AccessModelError> {
        for model in self.models.iter() {
            model.access_edge(traversal, state, state_model)?;
        }
        Ok(())
    }
}
