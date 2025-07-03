use std::fmt::Display;

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

impl Display for Label {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Label::Vertex(vertex_id) => write!(f, "Vertex({})", vertex_id),
            Label::VertexWithIntState { vertex_id, state } => {
                write!(f, "VertexWithIntState({}, {:?})", vertex_id, state)
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
