use std::collections::HashMap;

use super::frontier_model_error::FrontierModelError;
use crate::{
    algorithm::search::{Direction, SearchTreeBranch},
    model::{
        network::{Edge, VertexId},
        state::{StateModel, StateVariable},
    },
};

/// Validates edge and traversal states. Provides an API for removing edges from
/// the frontier in a way that could be more efficient than modifying the [TraversalModel].
/// This may be desireable when a traversal model has complex cost logic but an edge
/// may not be traversable for this query, such as due to height restrictions.
///
/// [TraversalModel]: crate::model::traversal::traversal_model::TraversalModel
pub trait FrontierModel: Send + Sync {
    /// Validates an edge before allowing it to be added to the search frontier.
    ///
    /// # Arguments
    ///
    /// * `edge` - the edge to traverse
    /// * `state` - the state of the traversal at the beginning of this edge
    /// * `tree` - the search tree for this search
    /// * `direction` - search direction
    /// * `state_model` - provides operations on the state vector
    ///
    /// # Returns
    ///
    /// True if the edge is a valid part of the frontier, false otherwise
    fn valid_frontier(
        &self,
        edge: &Edge,
        state: &[StateVariable],
        tree: &HashMap<VertexId, SearchTreeBranch>,
        direction: &Direction,
        state_model: &StateModel,
    ) -> Result<bool, FrontierModelError>;

    /// Validates an edge independent of a search state, noting whether it
    /// is simply impassable with this FrontierModel configuration. Can be
    /// called by valid_frontier as a cheaper first-pass operation. Also
    /// used by MapModel during query map matching.
    ///
    /// # Arguments
    ///
    /// * `edge` - the edge to test for validity
    ///
    /// # Returns
    ///
    /// True if the edge is valid
    fn valid_edge(&self, edge: &Edge) -> Result<bool, FrontierModelError>;
}
