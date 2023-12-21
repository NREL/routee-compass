use crate::model::traversal::traversal_model::TraversalModel;
use crate::model::unit::DistanceUnit;
use crate::model::unit::BASE_DISTANCE_UNIT;
use crate::model::{
    property::{edge::Edge, vertex::Vertex},
    traversal::{
        state::{state_variable::StateVar, traversal_state::TraversalState},
        traversal_model_error::TraversalModelError,
    },
};
use crate::util::geo::haversine;

/// A simple traversal model that uses the edge distance as the cost of traversal.
pub struct DistanceTraversalModel {
    distance_unit: DistanceUnit,
}

impl DistanceTraversalModel {
    pub fn new(distance_unit: DistanceUnit) -> DistanceTraversalModel {
        DistanceTraversalModel { distance_unit }
    }
}

impl TraversalModel for DistanceTraversalModel {
    fn state_variable_names(&self) -> Vec<String> {
        vec![String::from("distance")]
    }

    fn initial_state(&self) -> TraversalState {
        vec![StateVar(0.0)]
    }

    fn serialize_state(&self, state: &TraversalState) -> serde_json::Value {
        let total_distance = state[0].0;
        serde_json::json!({
            "distance": total_distance
        })
    }

    fn serialize_state_info(&self, _state: &TraversalState) -> serde_json::Value {
        serde_json::json!({
            "distance_unit": self.distance_unit
        })
    }

    fn traverse_edge(
        &self,
        _src: &Vertex,
        edge: &Edge,
        _dst: &Vertex,
        state: &TraversalState,
    ) -> Result<TraversalState, TraversalModelError> {
        let distance = BASE_DISTANCE_UNIT.convert(edge.distance, self.distance_unit);
        let mut updated_state = state.clone();
        updated_state[0] = state[0] + StateVar::from(distance);
        Ok(updated_state)
    }

    fn access_edge(
        &self,
        _v1: &Vertex,
        _src: &Edge,
        _v2: &Vertex,
        _dst: &Edge,
        _v3: &Vertex,
        _state: &TraversalState,
    ) -> Result<Option<TraversalState>, TraversalModelError> {
        Ok(None)
    }

    fn estimate_traversal(
        &self,
        src: &Vertex,
        dst: &Vertex,
        state: &TraversalState,
    ) -> Result<TraversalState, TraversalModelError> {
        let distance =
            haversine::coord_distance(src.coordinate, dst.coordinate, self.distance_unit)
                .map_err(TraversalModelError::NumericError)?;
        let mut updated_state = state.clone();
        updated_state[0] = state[0] + StateVar::from(distance);
        Ok(updated_state)
    }
}
