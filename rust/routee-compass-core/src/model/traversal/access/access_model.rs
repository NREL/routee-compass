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
    /// These arguments appear in the network as:
    /// `(v1) -[prev]-> (v2) -[next]-> (v3)`
    /// Where `next` is the edge we want to access.
    ///
    /// # Arguments
    ///
    /// * `v1` - src of previous edge
    /// * `src` - previous edge
    /// * `v2` - src vertex of the next edge
    /// * `dst` - edge we are determining the cost to access
    /// * `v3` - dst vertex of the next edge
    /// * `state` - state of the search at the beginning of the dst edge
    ///
    /// # Returns
    ///
    /// Either an optional access result or an error. if there are no
    /// state updates due to access, None is returned.
    fn access_edge(
        &self,
        v1: &Vertex,
        src: &Edge,
        v2: &Vertex,
        dst: &Edge,
        v3: &Vertex,
        state: &TraversalState,
    ) -> Result<Option<TraversalState>, TraversalModelError>;
}
