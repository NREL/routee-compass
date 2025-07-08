use std::fmt::Display;

use allocative::Allocative;
use serde::Serialize;

use crate::model::network::VertexId;

#[derive(PartialEq, Eq, Hash, Debug, Clone, Serialize, Allocative)]
pub enum Label {
    Vertex(VertexId),
    VertexWithIntState {
        vertex_id: VertexId,
        state: usize,
    },
    VertexWithIntStateVec {
        vertex_id: VertexId,
        state: Vec<usize>,
    },
}

impl Label {
    pub fn vertex_id(&self) -> VertexId {
        match self {
            Label::Vertex(vertex_id) => *vertex_id,
            Label::VertexWithIntState { vertex_id, .. } => *vertex_id,
            Label::VertexWithIntStateVec { vertex_id, .. } => *vertex_id,
        }
    }
}

impl Display for Label {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Label::Vertex(vertex_id) => write!(f, "Vertex({})", vertex_id),
            Label::VertexWithIntState { vertex_id, state } => {
                write!(f, "VertexWithIntState({}, {})", vertex_id, state)
            }
            Label::VertexWithIntStateVec { vertex_id, state } => {
                write!(f, "VertexWithIntStateVec({}, {:?})", vertex_id, state)
            }
        }
    }
}
