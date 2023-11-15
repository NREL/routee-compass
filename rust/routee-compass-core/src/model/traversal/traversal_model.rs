use super::access_result::AccessResult;
use super::traversal_model_error::TraversalModelError;
use crate::model::cost::Cost;
use crate::model::property::{edge::Edge, vertex::Vertex};
use crate::model::traversal::state::traversal_state::TraversalState;
use crate::model::traversal::traversal_result::TraversalResult;

/// Dictates how state transitions occur and how to evaluate the costs
/// while traversing a graph in a search algorithm.
///
/// see the [`super::default`] module for implementations bundled with RouteE Compass:
///   - [DistanceModel]: uses Edge distances to find the route with the shortest distance
///   - [SpeedLookupModel]: retrieves link speeds via lookup from a file
///
/// [DistanceModel]: super::default::distance::DistanceModel
/// [SpeedLookupModel]: super::default::speed_lookup_model::SpeedLookupModel
pub trait TraversalModel: Send + Sync {
    /// Creates the initial state of a search. this should be a vector of
    /// accumulators.
    ///
    /// # Returns
    ///
    /// an initialized, zero-valued traversal state
    fn initial_state(&self) -> TraversalState;

    /// Calculates the cost of traversing an edge due to some traversal state.
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
    fn traversal_cost(
        &self,
        src: &Vertex,
        edge: &Edge,
        dst: &Vertex,
        state: &TraversalState,
    ) -> Result<TraversalResult, TraversalModelError>;

    /// Calculates a cost estimate for traversing between a source and destination
    /// vertex without actually doing the work of traversing the edges.
    /// This estimate is used in search algorithms such as a-star algorithm, where
    /// the estimate is used to inform search order.
    ///
    /// # Arguments
    ///
    /// * `src` - source vertex
    /// * `dst` - destination vertex
    /// * `state` - state of the search at the beginning of this edge
    ///
    /// # Returns
    ///
    /// Either a cost estimate or an error.
    fn cost_estimate(
        &self,
        src: &Vertex,
        dst: &Vertex,
        state: &TraversalState,
    ) -> Result<Cost, TraversalModelError>;

    /// Calculates the cost of accessing some destination edge when coming
    /// from some previous edge.
    ///
    /// This implementation is provided as a default implementation, since
    /// edge access costs are not required for completing a search.
    ///
    /// These arguments appear in the network as:
    /// `(v1) -[prev]-> (v2) -[next]-> (v3)`
    /// Where `next` is the edge we want to access.
    ///
    /// # Arguments
    ///
    /// * `_v1` - src of previous edge
    /// * `prev` - previous edge
    /// * `_v2` - src vertex of the next edge
    /// * `next` - edge we are determining the cost to access
    /// * `_v3` - dst vertex of the next edge
    ///
    /// # Returns
    ///
    /// Either an access result or an error.
    fn access_cost(
        &self,
        _v1: &Vertex,
        _src: &Edge,
        _v2: &Vertex,
        _dst: &Edge,
        _v3: &Vertex,
        _state: &TraversalState,
    ) -> Result<AccessResult, TraversalModelError> {
        Ok(AccessResult::no_cost())
    }

    /// Evaluates whether the search state has reached some termination condition.
    /// For example, if the max travel time is 40 minutes, this function can return
    /// `true` to force the search to end.
    ///
    /// This default implementation can be overridden. It simply never adds any additional
    /// termination conditions.
    ///
    /// # Arguments
    ///
    /// * `state` - the search state
    ///
    /// # Returns
    ///
    /// `false` if the search can continue, `true` if the search should end, or, an error
    fn terminate_search(&self, _state: &TraversalState) -> Result<bool, TraversalModelError> {
        Ok(false)
    }

    /// Serializes the traversal state into a JSON value.
    ///
    /// This default implementation can be overwritten to write stateful information
    /// to route and summary outputs.
    ///
    /// # Arguments
    ///
    /// * `state` - the state to serialize
    ///
    /// # Returns
    ///
    /// A JSON serialized version of the state. This does not need to include
    /// additional details such as the units (kph, hours, etc), which can be
    /// summarized in the serialize_state_info method.
    fn serialize_state(&self, _state: &TraversalState) -> serde_json::Value {
        serde_json::json!({})
    }

    /// Serializes other information about a traversal state as a JSON value.
    ///
    /// This default implementation can be overwritten to write stateful information
    /// to route and summary outputs.
    ///
    /// # Arguments
    ///
    /// * `state` - the state to serialize information from
    ///
    /// # Returns
    ///
    /// JSON containing information such as the units (kph, hours, etc) or other
    /// traversal info (charge events, days traveled, etc)
    fn serialize_state_info(&self, _state: &TraversalState) -> serde_json::Value {
        serde_json::json!({})
    }

    /// Serialization function called by Compass output processing code that
    /// writes both the state and the state info to a JSON value.
    ///
    /// # Arguments
    ///
    /// * `state` - the state to serialize information from
    ///
    /// # Returns
    ///
    /// JSON containing the state values and info described in `serialize_state`
    /// and `serialize_state_info`.
    fn serialize_state_with_info(&self, state: &TraversalState) -> serde_json::Value {
        use serde_json::Value as Json;
        let mut summary = self.serialize_state(state);
        let summary_info = match self.serialize_state_info(state) {
            Json::Null => serde_json::Map::new().into_iter(),
            Json::Object(m) => m.into_iter(),
            other => {
                // this is just a fallback implementation in case TraversalModel builders something
                // other than what we expected
                let mut m = serde_json::Map::new();
                m.insert(String::from("info"), other);
                m.into_iter()
            }
        };
        for (k, v) in summary_info {
            summary[k] = v;
        }
        summary
    }
}
