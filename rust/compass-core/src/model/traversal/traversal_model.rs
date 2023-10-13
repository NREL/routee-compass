use serde_json::Map;

use super::access_result::AccessResult;
use super::traversal_model_error::TraversalModelError;
use crate::model::cost::cost::Cost;
use crate::model::property::{edge::Edge, vertex::Vertex};
use crate::model::traversal::state::traversal_state::TraversalState;
use crate::model::traversal::traversal_result::TraversalResult;

pub trait TraversalModel: Send + Sync {
    fn initial_state(&self) -> TraversalState;
    fn traversal_cost(
        &self,
        src: &Vertex,
        edge: &Edge,
        dst: &Vertex,
        state: &TraversalState,
    ) -> Result<TraversalResult, TraversalModelError>;
    fn cost_estimate(
        &self,
        src: &Vertex,
        dst: &Vertex,
        state: &TraversalState,
    ) -> Result<Cost, TraversalModelError>;
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
    fn terminate_search(&self, _state: &TraversalState) -> Result<bool, TraversalModelError> {
        Ok(false)
    }
    fn serialize_state(&self, _state: &TraversalState) -> serde_json::Value {
        serde_json::json!({})
    }
    fn serialize_state_info(&self, _state: &TraversalState) -> serde_json::Value {
        serde_json::json!({})
    }
    fn serialize_state_with_units(&self, state: &TraversalState) -> serde_json::Value {
        use serde_json::Value as Json;
        let mut summary = self.serialize_state(&state);
        let summary_info = match self.serialize_state_info(&state) {
            Json::Null => serde_json::Map::new().into_iter(),
            Json::Object(m) => m.into_iter(),
            other => {
                // this is just a fallback implementation in case TraversalModel builders return something
                // other than what we expected
                let mut m = serde_json::Map::new();
                m.insert(String::from("info"), other);
                m.into_iter()
            }
        };
        for (k, v) in summary_info {
            summary[k] = v;
        }
        return summary;
    }
}
