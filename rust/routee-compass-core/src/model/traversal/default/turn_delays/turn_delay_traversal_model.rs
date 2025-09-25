use uom::{si::f64::Time, ConstZero};

use super::TurnDelayTraversalModelEngine;
use crate::{
    algorithm::search::SearchTree,
    model::{
        network::{Edge, Vertex},
        state::{StateModel, StateVariable, StateVariableConfig},
        traversal::{default::fieldname, TraversalModel, TraversalModelError},
        unit::TimeUnit,
    },
};
use std::sync::Arc;

pub struct TurnDelayTraversalModel {
    pub engine: Arc<TurnDelayTraversalModelEngine>,
}

impl TurnDelayTraversalModel {
    pub const EDGE_TIME: &'static str = "edge_time";
    pub const TRIP_TIME: &'static str = "trip_time";

    pub fn new(engine: Arc<TurnDelayTraversalModelEngine>) -> Self {
        TurnDelayTraversalModel { engine }
    }
}

impl TraversalModel for TurnDelayTraversalModel {
    fn name(&self) -> String {
        "Turn Delay Traversal Model".to_string()
    }

    fn input_features(&self) -> Vec<crate::model::state::InputFeature> {
        vec![]
    }

    fn output_features(&self) -> Vec<(String, StateVariableConfig)> {
        vec![
            (
                String::from(fieldname::EDGE_TIME),
                StateVariableConfig::Time {
                    initial: Time::ZERO,
                    accumulator: false,
                    output_unit: Some(TimeUnit::Seconds),
                },
            ),
            (
                String::from(fieldname::TRIP_TIME),
                StateVariableConfig::Time {
                    initial: Time::ZERO,
                    accumulator: true,
                    output_unit: Some(TimeUnit::Seconds),
                },
            ),
        ]
    }

    fn traverse_edge(
        &self,
        traversal: (&Vertex, &Edge, &Vertex),
        state: &mut Vec<StateVariable>,
        tree: &SearchTree,
        state_model: &StateModel,
    ) -> Result<(), TraversalModelError> {
        let (src, edge, _) = traversal;
        let prev_opt = tree
            .backtrack_with_depth(src.vertex_id, 1)
            .map_err(|e| {
                TraversalModelError::TraversalModelFailure(format!(
                    "while applying turn delays, {e}"
                ))
            })?
            .first()
            .map(|et| et.edge_id);
        if let Some(prev) = prev_opt {
            let delay = self.engine.get_delay(prev, edge.edge_id)?;
            state_model.set_time(state, Self::EDGE_TIME, &delay)?;
            state_model.add_time(state, Self::TRIP_TIME, &delay)?;
        }

        Ok(())
    }

    fn estimate_traversal(
        &self,
        _od: (&Vertex, &Vertex),
        _state: &mut Vec<StateVariable>,
        _tree: &SearchTree,
        _state_model: &StateModel,
    ) -> Result<(), TraversalModelError> {
        Ok(())
    }
}
