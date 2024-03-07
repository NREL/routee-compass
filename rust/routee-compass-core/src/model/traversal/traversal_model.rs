use super::state::state_variable::StateVar;
use super::traversal_model_error::TraversalModelError;
use crate::model::property::{edge::Edge, vertex::Vertex};
use crate::model::state::state_feature::StateFeature;
use crate::model::traversal::state::traversal_state::TraversalState;

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
        src: &Vertex,
        edge: &Edge,
        dst: &Vertex,
        state: &mut Vec<StateVar>,
    ) -> Result<(), TraversalModelError>;

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
        state: &mut Vec<StateVar>,
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
        src: &Vertex,
        dst: &Vertex,
        state: &mut Vec<StateVar>,
    ) -> Result<(), TraversalModelError>;

    // /// Serializes the traversal state into a JSON value.
    // ///
    // /// This default implementation can be overwritten to write stateful information
    // /// to route and summary outputs.
    // ///
    // /// # Arguments
    // ///
    // /// * `state` - the state to serialize
    // ///
    // /// # Returns
    // ///
    // /// A JSON serialized version of the state. This does not need to include
    // /// additional details such as the units (kph, hours, etc), which can be
    // /// summarized in the serialize_state_info method.
    // fn serialize_state(&self, _state: &[StateVar]) -> serde_json::Value {
    //     serde_json::json!({})
    // }

    // /// Serializes other information about a traversal state as a JSON value.
    // ///
    // /// This default implementation can be overwritten to write stateful information
    // /// to route and summary outputs.
    // ///
    // /// # Arguments
    // ///
    // /// * `state` - the state to serialize information from
    // ///
    // /// # Returns
    // ///
    // /// JSON containing information such as the units (kph, hours, etc) or other
    // /// traversal info (charge events, days traveled, etc)
    // fn serialize_state_info(&self, _state: &[StateVar]) -> serde_json::Value {
    //     serde_json::json!({})
    // }

    // /// Serialization function called by Compass output processing code that
    // /// writes both the state and the state info to a JSON value.
    // ///
    // /// # Arguments
    // ///
    // /// * `state` - the state to serialize information from
    // ///
    // /// # Returns
    // ///
    // /// JSON containing the state values and info described in `serialize_state`
    // /// and `serialize_state_info`.
    // fn serialize_state_with_info(&self, state: &[StateVar]) -> serde_json::Value {
    //     use serde_json::Value as Json;
    //     let mut summary = self.serialize_state(state);
    //     let summary_info = match self.serialize_state_info(state) {
    //         Json::Null => serde_json::Map::new().into_iter(),
    //         Json::Object(m) => m.into_iter(),
    //         other => {
    //             // this is just a fallback implementation in case TraversalModel builders something
    //             // other than what we expected
    //             let mut m = serde_json::Map::new();
    //             m.insert(String::from("info"), other);
    //             m.into_iter()
    //         }
    //     };
    //     for (k, v) in summary_info {
    //         summary[k] = v;
    //     }
    //     summary
    // }
}
