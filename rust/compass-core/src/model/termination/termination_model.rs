use crate::model::{property::edge::Edge, traversal::state::traversal_state::TraversalState};

use super::termination_model_error::TerminationModelError;

pub trait TerminationModel: Send + Sync {
    fn terminate_search(
        &self,
        edge: &Edge,
        state: &TraversalState,
        iterations: u64,
    ) -> Result<bool, TerminationModelError>;

    fn summarize_termination(
        &self,
        edge: &Edge,
        state: &TraversalState,
        iterations: u64,
    ) -> Result<serde_json::Value, TerminationModelError>;
}
