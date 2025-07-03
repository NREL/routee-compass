use allocative::Allocative;
use serde::Serialize;

use crate::model::{
    label::label_model_error::LabelModelError,
    network::VertexId,
    state::{StateModel, StateVariable},
};

#[derive(PartialEq, Eq, Hash, Debug, Clone, Serialize, Allocative)]
pub enum Label {
    Vertex(VertexId),
    VertexWithIntState {
        vertex_id: VertexId,
        state: Vec<u64>,
    },
}

impl Label {
    pub fn vertex_id(&self) -> VertexId {
        match self {
            Label::Vertex(vertex_id) => *vertex_id,
            Label::VertexWithIntState { vertex_id, .. } => *vertex_id,
        }
    }
}

impl ToString for Label {
    fn to_string(&self) -> String {
        match self {
            Label::Vertex(vertex_id) => format!("Vertex({})", vertex_id.0),
            Label::VertexWithIntState { vertex_id, state } => {
                format!("VertexWithIntState({}, {:?})", vertex_id.0, state)
            }
        }
    }
}
pub trait LabelModel: Send + Sync {
    fn label_from_state(
        &self,
        vertex_id: VertexId,
        state: &[StateVariable],
        state_model: &StateModel,
    ) -> Result<Label, LabelModelError>;
}
