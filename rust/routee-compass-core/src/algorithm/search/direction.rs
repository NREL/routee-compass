use super::{
    edge_traversal::EdgeTraversal, search_error::SearchError, search_instance::SearchInstance,
};
use crate::model::graph::{Edge, EdgeId, GraphError, VertexId};
use crate::model::traversal::state::state_variable::StateVar;
use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Serialize, Deserialize, Default)]
#[serde(rename = "snake_case")]
pub enum Direction {
    #[default]
    Forward,
    Reverse,
}

impl Direction {
    pub fn get_incident_edges<'a>(
        &'a self,
        vertex_id: &VertexId,
        si: &'a SearchInstance,
    ) -> Result<Box<dyn Iterator<Item = &EdgeId> + 'a>, GraphError> {
        match self {
            Direction::Forward => si.directed_graph.out_edges_iter(*vertex_id),
            Direction::Reverse => si.directed_graph.in_edges_iter(*vertex_id),
        }
    }

    pub fn tree_key_vertex_id(&self, edge: &Edge) -> VertexId {
        match self {
            Direction::Forward => edge.dst_vertex_id,
            Direction::Reverse => edge.src_vertex_id,
        }
    }

    pub fn terminal_vertex_id(&self, edge: &Edge) -> VertexId {
        match self {
            Direction::Forward => edge.src_vertex_id,
            Direction::Reverse => edge.dst_vertex_id,
        }
    }

    pub fn perform_edge_traversal(
        &self,
        edge_id: EdgeId,
        last_edge_id: Option<EdgeId>,
        start_state: &[StateVar],
        si: &SearchInstance,
    ) -> Result<EdgeTraversal, SearchError> {
        match self {
            Direction::Forward => {
                EdgeTraversal::forward_traversal(edge_id, last_edge_id, start_state, si)
            }
            Direction::Reverse => {
                EdgeTraversal::reverse_traversal(edge_id, last_edge_id, start_state, si)
            }
        }
    }
}
