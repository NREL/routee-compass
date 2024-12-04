use super::frontier_model_error::FrontierModelError;
use crate::model::{
    network::Edge, state::state_model::StateModel, traversal::state::state_variable::StateVar,
};

/// Validates edge and traversal states. Provides an API for removing edges from
/// the frontier in a way that could be more efficient than modifying the [TraversalModel].
/// This may be desireable when a traversal model has complex cost logic but an edge
/// may not be traversable for this query, such as due to height restrictions.
///
/// [TraversalModel]: crate::model::traversal::traversal_model::TraversalModel
pub trait FrontierModel: Send + Sync {
    /// Validates an edge before allowing it to be added to the frontier.
    ///
    /// # Arguments
    ///
    /// * `edge` - the edge to traverse
    /// * `state` - the state of the traversal at the beginning of this edge
    /// * `previous_edge` - the edge that was traversed to reach this edge
    ///
    /// # Returns
    ///
    /// True if the edge is valid, false otherwise; Or, an error from processing
    fn valid_frontier(
        &self,
        _edge: &Edge,
        _state: &[StateVar],
        _previous_edge: Option<&Edge>,
        _state_model: &StateModel,
    ) -> Result<bool, FrontierModelError> {
        Ok(true)
    }
}
