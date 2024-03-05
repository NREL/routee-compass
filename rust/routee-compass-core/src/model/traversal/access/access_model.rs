use crate::model::{
    property::{edge::Edge, vertex::Vertex},
    traversal::{
        state::traversal_state::TraversalState, traversal_model_error::TraversalModelError,
    },
};

pub trait AccessModel {
    /// Updates the traversal state by accessing some destination edge
    /// when coming from some previous edge.
    ///
    /// The traversal argument represents a set of vertices and
    /// edges connected in the network:
    /// `(v1) -[prev]-> (v2) -[next]-> (v3)`
    /// Where `next` is the edge we want to access.
    ///
    /// # Arguments
    ///
    /// * `traversal` - the vertex/edge traversal
    /// * `state` - state of the search at the beginning of the dst edge
    /// * `state_variable_indices` - the names and indices of state variables
    ///
    /// # Returns
    ///
    /// Either an optional access result or an error. if there are no
    /// state updates due to access, None is returned.
    fn access_edge(
        &self,
        traversal: (&Vertex, &Edge, &Vertex, &Edge, &Vertex),
        state: &TraversalState,
        state_variable_indices: Vec<(String, usize)>,
    ) -> Result<Option<TraversalState>, TraversalModelError>;
}
