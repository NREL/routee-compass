use std::sync::Arc;

use crate::model::access::{
    access_model::AccessModel, access_model_builder::AccessModelBuilder,
    access_model_service::AccessModelService,
};

#[derive(Clone, Debug)]
pub struct NoAccessModel {}

impl AccessModel for NoAccessModel {
    fn state_features(&self) -> Vec<(String, crate::model::state::state_feature::StateFeature)> {
        vec![]
    }

    fn access_edge(
        &self,
        _traversal: (
            &crate::model::property::vertex::Vertex,
            &crate::model::property::edge::Edge,
            &crate::model::property::vertex::Vertex,
            &crate::model::property::edge::Edge,
            &crate::model::property::vertex::Vertex,
        ),
        _state: &mut Vec<crate::model::traversal::state::state_variable::StateVar>,
        _state_model: &crate::model::state::state_model::StateModel,
    ) -> Result<(), crate::model::access::access_model_error::AccessModelError> {
        Ok(())
    }
}

impl AccessModelService for NoAccessModel {
    fn build(
        &self,
        _query: &serde_json::Value,
    ) -> Result<
        std::sync::Arc<dyn AccessModel>,
        crate::model::access::access_model_error::AccessModelError,
    > {
        let model: Arc<dyn AccessModel> = Arc::new(self.clone());
        Ok(model)
    }
}

impl AccessModelBuilder for NoAccessModel {
    fn build(
        &self,
        _parameters: &serde_json::Value,
    ) -> Result<
        Arc<dyn AccessModelService>,
        crate::model::access::access_model_error::AccessModelError,
    > {
        let service: Arc<dyn AccessModelService> = Arc::new(self.clone());
        Ok(service)
    }
}
