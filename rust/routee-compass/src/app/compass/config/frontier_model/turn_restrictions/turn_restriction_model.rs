use routee_compass_core::model::{
    frontier::{frontier_model::FrontierModel, frontier_model_error::FrontierModelError},
    network::Edge,
    state::state_model::StateModel,
    traversal::state::state_variable::StateVar,
};
use std::sync::Arc;

use super::turn_restriction_service::{RestrictedEdgePair, TurnRestrictionFrontierService};

pub struct TurnRestrictionFrontierModel {
    pub service: Arc<TurnRestrictionFrontierService>,
}

impl FrontierModel for TurnRestrictionFrontierModel {
    fn valid_frontier(
        &self,
        edge: &Edge,
        _state: &[StateVar],
        previous_edge: Option<&Edge>,
        _state_model: &StateModel,
    ) -> Result<bool, FrontierModelError> {
        match previous_edge {
            None => Ok(true),
            Some(previous_edge) => {
                let edge_pair = RestrictedEdgePair {
                    prev_edge_id: previous_edge.edge_id,
                    next_edge_id: edge.edge_id,
                };
                if self.service.restricted_edge_pairs.contains(&edge_pair) {
                    return Ok(false);
                }
                Ok(true)
            }
        }
    }
}
