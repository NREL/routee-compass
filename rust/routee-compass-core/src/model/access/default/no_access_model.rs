use crate::model::{
    access::{AccessModel, AccessModelBuilder, AccessModelService},
    network::{Edge, Vertex},
};
use std::sync::Arc;

#[derive(Clone, Debug)]
pub struct NoAccessModel {}

impl AccessModel for NoAccessModel {
    fn state_features(&self) -> Vec<(String, crate::model::state::StateFeature)> {
        vec![]
    }

    fn access_edge(
        &self,
        _: (&Vertex, &Edge, &Vertex, &Edge, &Vertex),
        _: &mut Vec<crate::model::state::StateVariable>,
        _: &crate::model::state::StateModel,
    ) -> Result<(), crate::model::access::AccessModelError> {
        Ok(())
    }
}

impl AccessModelService for NoAccessModel {
    fn build(
        &self,
        _query: &serde_json::Value,
    ) -> Result<std::sync::Arc<dyn AccessModel>, crate::model::access::AccessModelError> {
        let model: Arc<dyn AccessModel> = Arc::new(self.clone());
        Ok(model)
    }
}

impl AccessModelBuilder for NoAccessModel {
    fn build(
        &self,
        _parameters: &serde_json::Value,
    ) -> Result<Arc<dyn AccessModelService>, crate::model::access::AccessModelError> {
        let service: Arc<dyn AccessModelService> = Arc::new(self.clone());
        Ok(service)
    }
}
