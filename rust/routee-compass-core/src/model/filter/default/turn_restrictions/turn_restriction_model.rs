use crate::model::{
    filter::{FilterModel, FilterModelError},
    network::Edge,
    state::{StateModel, StateVariable},
};
use std::sync::Arc;

use super::turn_restriction_service::{RestrictedEdgePair, TurnRestrictionFrontierService};

pub struct TurnRestrictionFilterModel {
    pub service: Arc<TurnRestrictionFrontierService>,
}

impl FilterModel for TurnRestrictionFilterModel {
    fn valid_frontier(
        &self,
        edge: &Edge,
        previous_edge: Option<&Edge>,
        _state: &[StateVariable],
        _state_model: &StateModel,
    ) -> Result<bool, FilterModelError> {
        match previous_edge {
            Some(previous_edge) => {
                let edge_pair = RestrictedEdgePair {
                    prev_edge_id: previous_edge.edge_id,
                    next_edge_id: edge.edge_id,
                };
                if self.service.restricted_edge_pairs.contains(&edge_pair) {
                    Ok(false)
                } else {
                    Ok(true)
                }
            }
            None => Ok(true),
        }
    }

    fn valid_edge(&self, _edge: &Edge) -> Result<bool, FilterModelError> {
        Ok(true)
    }
}
