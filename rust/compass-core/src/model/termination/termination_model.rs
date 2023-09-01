use super::termination_model_error::TerminationModelError;
use crate::model::{property::edge::Edge, traversal::state::traversal_state::TraversalState};
use std::time::Instant;

pub trait TerminationModel: Send + Sync {
    fn terminate_search(
        &self,
        edge: &Edge,
        state: &TraversalState,
        start_time: Instant,
        iterations: u64,
    ) -> Result<bool, TerminationModelError>;

    fn summarize_termination(
        &self,
        edge: &Edge,
        state: &TraversalState,
        start_time: Instant,
        iterations: u64,
    ) -> Result<Option<String>, TerminationModelError>;
}
