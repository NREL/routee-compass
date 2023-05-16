use std::fmt::Display;

use crate::{algorithm::search::edge_traversal::EdgeTraversal, model::graph::vertex_id::VertexId};

#[derive(Clone)]
pub struct AStarTraversal<S> {
    pub terminal_vertex: VertexId,
    pub edge_traversal: EdgeTraversal<S>,
}

impl<S: Display> Display for AStarTraversal<S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "-[acost:{} tcost:{} state:{}]-> ({})",
            self.edge_traversal.access_cost,
            self.edge_traversal.traversal_cost,
            self.edge_traversal.result_state,
            self.terminal_vertex
        )
    }
}
