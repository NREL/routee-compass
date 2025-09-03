use super::SearchInstance2;
use crate::model::network::{Edge, EdgeId, EdgeListId, VertexId};
use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Serialize, Deserialize, Default, Debug)]
#[serde(rename = "snake_case")]
pub enum Direction {
    #[default]
    Forward,
    Reverse,
}

impl Direction {
    pub fn get_incident_edges<'a>(
        &'a self,
        vertex_id: &'a VertexId,
        si: &'a SearchInstance2,
    ) -> Box<dyn Iterator<Item = (EdgeListId, EdgeId)> + 'a> {
        match self {
            Direction::Forward => si.graph.out_edges_iter(vertex_id),
            Direction::Reverse => si.graph.in_edges_iter(vertex_id),
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

    // pub fn perform_edge_traversal(
    //     &self,
    //     edge_id: EdgeId,
    //     last_edge_id: Option<EdgeId>,
    //     start_state: &[StateVariable],
    //     si: &SearchInstance2,
    // ) -> Result<EdgeTraversal, SearchError> {
    //     match self {
    //         Direction::Forward => {
    //             EdgeTraversal::forward_traversal(edge_id, last_edge_id, start_state, si)
    //         }
    //         Direction::Reverse => {
    //             EdgeTraversal::reverse_traversal(edge_id, last_edge_id, start_state, si)
    //         }
    //     }
    // }
}
