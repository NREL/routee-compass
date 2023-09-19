use crate::model::traversal::traversal_result::TraversalResult;
use crate::model::{
    cost::cost::Cost,
    property::{edge::Edge, vertex::Vertex},
    traversal::{
        state::{state_variable::StateVar, traversal_state::TraversalState},
        traversal_model::TraversalModel,
        traversal_model_error::TraversalModelError,
    },
};
use crate::util::geo::haversine::coord_distance_km;
use crate::util::unit::DistanceUnit;
use crate::util::unit::BASE_DISTANCE_UNIT;

/// A simple traversal model that uses the edge distance as the cost of traversal.
pub struct DistanceModel {
    distance_unit: DistanceUnit,
}

impl DistanceModel {
    pub fn new() -> DistanceModel {
        DistanceModel {
            distance_unit: DistanceUnit::Meters,
        }
    }
}

impl TraversalModel for DistanceModel {
    fn initial_state(&self) -> TraversalState {
        vec![StateVar(0.0)]
    }
    fn traversal_cost(
        &self,
        _src: &Vertex,
        edge: &Edge,
        _dst: &Vertex,
        state: &Vec<StateVar>,
    ) -> Result<TraversalResult, TraversalModelError> {
        let distance = BASE_DISTANCE_UNIT.convert(edge.distance, self.distance_unit.clone());
        let mut updated_state = state.clone();
        updated_state[0] = state[0] + StateVar::from(distance);
        let result = TraversalResult {
            total_cost: Cost::from(distance),
            updated_state,
        };
        Ok(result)
    }
    fn cost_estimate(
        &self,
        src: &Vertex,
        dst: &Vertex,
        state: &TraversalState,
    ) -> Result<Cost, TraversalModelError> {
        return coord_distance_km(src.coordinate, dst.coordinate)
            .map(|d| Cost::from(d))
            .map_err(TraversalModelError::NumericError);
    }
    fn summary(&self, state: &TraversalState) -> serde_json::Value {
        let total_distance = state[0].0;
        serde_json::json!({
            "distance": total_distance,
            "distance_unit": self.distance_unit
        })
    }
}
