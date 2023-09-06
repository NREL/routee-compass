use crate::model::{property::edge::Edge, traversal::state::traversal_state::TraversalState};

use super::frontier_model_error::FrontierModelError;

pub trait FrontierModel: Send + Sync {
    fn valid_frontier(
        &self,
        edge: &Edge,
        state: &TraversalState,
    ) -> Result<bool, FrontierModelError> {
        Ok(true)
    }
}
