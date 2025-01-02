use super::traversal_model_error::TraversalModelError;
use crate::model::network::{Edge, Vertex};
use crate::model::state::StateFeature;
use crate::model::state::StateModel;
use crate::model::state::StateVariable;

/// Dictates how state transitions occur while traversing a graph in a search algorithm.
///
/// see the [`super::default`] module for implementations bundled with RouteE Compass:
///   - [DistanceModel]: uses Edge distances to find the route with the shortest distance
///   - [SpeedLookupModel]: retrieves link speeds via lookup from a file
///
/// [DistanceModel]: super::default::distance::DistanceModel
/// [SpeedLookupModel]: super::default::speed_lookup_model::SpeedLookupModel
pub trait TraversalModel: Send + Sync {
    /// lists the state variables expected by this traversal model that are not
    /// defined on the base configuration. for example, if this traversal model
    /// has state variables that differ based on the query, they can be injected
    /// into the state model by listing them here.
    fn state_features(&self) -> Vec<(String, StateFeature)>;

    /// Updates the traversal state by traversing an edge.
    ///
    /// # Arguments
    ///
    /// * `src` - source vertex
    /// * `edge` - edge to traverse
    /// * `dst` - destination vertex
    /// * `state` - state of the search at the beginning of this edge
    ///
    /// # Returns
    ///
    /// Either a traversal result or an error.
    fn traverse_edge(
        &self,
        trajectory: (&Vertex, &Edge, &Vertex),
        state: &mut Vec<StateVariable>,
        state_model: &StateModel,
    ) -> Result<(), TraversalModelError>;

    /// Estimates the traversal state by traversing between two vertices without
    /// performing any graph traversals.
    ///
    /// # Arguments
    ///
    /// * `src` - source vertex
    /// * `dst` - destination vertex
    /// * `state` - state of the search at the source vertex
    ///
    /// # Returns
    ///
    /// Either a traversal result or an error.
    fn estimate_traversal(
        &self,
        od: (&Vertex, &Vertex),
        state: &mut Vec<StateVariable>,
        state_model: &StateModel,
    ) -> Result<(), TraversalModelError>;
}
