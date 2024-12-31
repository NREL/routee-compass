use routee_compass_core::{
    algorithm::search::{Direction, SearchTreeBranch},
    model::{
        frontier::{FrontierModel, FrontierModelError},
        network::{Edge, VertexId},
        state::{StateModel, StateVariable},
    },
};
use std::{collections::HashMap, sync::Arc};

use super::turn_restriction_service::{RestrictedEdgePair, TurnRestrictionFrontierService};

pub struct TurnRestrictionFrontierModel {
    pub service: Arc<TurnRestrictionFrontierService>,
}

impl FrontierModel for TurnRestrictionFrontierModel {
    fn valid_frontier(
        &self,
        edge: &Edge,
        _state: &[StateVariable],
        tree: &HashMap<VertexId, SearchTreeBranch>,
        direction: &Direction,
        _state_model: &StateModel,
    ) -> Result<bool, FrontierModelError> {
        let previous_edge = match direction {
            Direction::Forward => tree.get(&edge.src_vertex_id),
            Direction::Reverse => tree.get(&edge.dst_vertex_id),
        };
        match previous_edge {
            None => Ok(true),
            Some(previous_edge) => {
                let edge_pair = RestrictedEdgePair {
                    prev_edge_id: previous_edge.edge_traversal.edge_id,
                    next_edge_id: edge.edge_id,
                };
                if self.service.restricted_edge_pairs.contains(&edge_pair) {
                    return Ok(false);
                }
                Ok(true)
            }
        }
    }

    fn valid_edge(&self, _edge: &Edge) -> Result<bool, FrontierModelError> {
        Ok(true)
    }
}
