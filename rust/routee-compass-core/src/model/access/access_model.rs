use super::access_model_error::AccessModelError;
use crate::model::{
    property::{edge::Edge, vertex::Vertex},
    state::{state_feature::StateFeature, state_model::StateModel},
    traversal::state::state_variable::StateVar,
};

pub trait AccessModel {
    /// lists the state variables expected by this access model that are not
    /// defined on the base configuration. for example, if this access model
    /// has state variables that differ based on the query, they can be injected
    /// into the state model by listing them here.
    fn state_features(&self) -> Vec<(String, StateFeature)>;

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
        state: &mut Vec<StateVar>,
        state_model: &StateModel,
    ) -> Result<(), AccessModelError>;
}
