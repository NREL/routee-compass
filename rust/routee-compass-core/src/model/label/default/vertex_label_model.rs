use std::sync::Arc;

use crate::model::{
    label::{
        label_enum::Label, label_model::LabelModel, label_model_builder::LabelModelBuilder,
        label_model_error::LabelModelError, label_model_service::LabelModelService,
    },
    network::VertexId,
    state::{StateModel, StateVariable},
};

pub struct VertexLabelModel;

impl LabelModel for VertexLabelModel {
    fn label_from_state(
        &self,
        vertex_id: VertexId,
        _state: &[StateVariable],
        _state_model: &StateModel,
    ) -> Result<Label, LabelModelError> {
        Ok(Label::Vertex(vertex_id))
    }
}

pub struct VertexLabelModelService;

impl LabelModelService for VertexLabelModelService {
    fn build(
        &self,
        _query: &serde_json::Value,
        _state_model: Arc<StateModel>,
    ) -> Result<Arc<dyn LabelModel>, LabelModelError> {
        Ok(Arc::new(VertexLabelModel))
    }
}

pub struct VertexLabelModelBuilder;

impl LabelModelBuilder for VertexLabelModelBuilder {
    fn build(
        &self,
        _parameters: &serde_json::Value,
    ) -> Result<Arc<dyn LabelModelService>, LabelModelError> {
        Ok(Arc::new(VertexLabelModelService))
    }
}
