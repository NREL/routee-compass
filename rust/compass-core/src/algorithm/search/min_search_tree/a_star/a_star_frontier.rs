use std::{hash::Hash, hash::Hasher};

use crate::model::{graph::edge_id::EdgeId, graph::vertex_id::VertexId};

#[derive(Clone, Eq, PartialEq)]
pub struct AStarFrontier<S> {
    pub vertex_id: VertexId,
    pub prev_edge_id: Option<EdgeId>,
    pub state: S,
}

impl<S> Hash for AStarFrontier<S> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.vertex_id.hash(state);
    }
}
