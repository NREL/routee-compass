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
use uom::si;

/// A simple traversal model that uses the edge distance as the cost of traversal.
pub struct DistanceModel {}

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
        let cost = edge.distance.get::<si::length::meter>();
        let mut updated_state = state.clone();
        updated_state[0] = state[0] + StateVar(cost);
        let result = TraversalResult {
            total_cost: Cost::from(cost),
            updated_state,
        };
        Ok(result)
    }
    fn cost_estimate(
        &self,
        src: Vertex,
        dst: Vertex,
        state: &TraversalState,
    ) -> Result<Cost, TraversalModelError> {
        return coord_distance_km(src.coordinate, dst.coordinate)
            .map(|d| Cost::from(d.value))
            .map_err(TraversalModelError::NumericError);
    }
    fn summary(&self, state: &TraversalState) -> serde_json::Value {
        let total_distance_meters = state[0].0;
        serde_json::json!({
            "distance_meters": total_distance_meters,
        })
    }
}
