use crate::model::traversal::traversal_result::TraversalResult;
use crate::model::{
    cost::cost::Cost,
    property::{edge::Edge, vertex::Vertex},
    traversal::{
        state::{traversal_state::TraversalState, state_variable::StateVar},
        traversal_model_error::TraversalModelError,
        traversal_model::TraversalModel,
    },
};
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
    fn summary(&self, state: &TraversalState) -> serde_json::Value {
        let total_distance_meters = state[0].0;
        serde_json::json!({
            "distance_meters": total_distance_meters,
        })
        
    }
}