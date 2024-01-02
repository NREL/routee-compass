use routee_compass_core::model::{
    frontier::{frontier_model::FrontierModel, frontier_model_error::FrontierModelError},
    property::edge::Edge,
    traversal::state::traversal_state::TraversalState,
};
use std::sync::Arc;

use super::turn_restriction_service::TurnRestrictionFrontierService;

pub struct TurnRestrictionFrontierModel {
    pub service: Arc<TurnRestrictionFrontierService>,
}

impl FrontierModel for TurnRestrictionFrontierModel {
    fn valid_frontier(
        &self,
        edge: &Edge,
        _state: &TraversalState,
        previous_edge: Option<&Edge>,
    ) -> Result<bool, FrontierModelError> {
        match previous_edge {
            None => Ok(true),
            Some(previous_edge) => {
                let edge_id_tuple = (previous_edge.edge_id, edge.edge_id);
                if self.service.restricted_edges.contains(&edge_id_tuple) {
                    return Ok(false);
                }
                Ok(true)
            }
        }
    }
}
