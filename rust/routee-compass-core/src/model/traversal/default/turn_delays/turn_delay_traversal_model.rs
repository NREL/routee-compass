use super::TurnDelayTraversalModelEngine;
use crate::{
    algorithm::search::SearchTree,
    model::{
        network::{Edge, Vertex},
        state::{StateModel, StateVariable},
        traversal::{TraversalModel, TraversalModelError},
    },
};
use std::sync::Arc;

pub struct TurnDelayTraversalModel {
    pub engine: Arc<TurnDelayTraversalModelEngine>,
    pub include_trip_time: bool,
    // Cached indices for performance
    edge_turn_delay_idx: usize,
    edge_time_idx: usize,
    trip_time_idx: Option<usize>,
}

impl TurnDelayTraversalModel {
    pub fn new(
        engine: Arc<TurnDelayTraversalModelEngine>,
        include_trip_time: bool,
        edge_turn_delay_idx: usize,
        edge_time_idx: usize,
        trip_time_idx: Option<usize>,
    ) -> TurnDelayTraversalModel {
        TurnDelayTraversalModel {
            engine,
            include_trip_time,
            edge_turn_delay_idx,
            edge_time_idx,
            trip_time_idx,
        }
    }
}

impl TraversalModel for TurnDelayTraversalModel {
    fn name(&self) -> String {
        "Turn Delay Traversal Model".to_string()
    }

    fn traverse_edge(
        &self,
        traversal: (&Vertex, &Edge, &Vertex),
        state: &mut Vec<StateVariable>,
        tree: &SearchTree,
        state_model: &StateModel,
    ) -> Result<(), TraversalModelError> {
        if tree.is_empty() {
            // we need a previous edge to complete a turn
            return Ok(());
        }
        let (src, edge, _) = traversal;

        // Efficiently get the incoming edge for the source vertex (previous edge in path)
        let prev_edge_id = match tree.get_incoming_edge(src.vertex_id) {
            Some(prev_traversal) => prev_traversal.edge_id,
            None => return Ok(()), // No previous edge (at root), no turn delay
        };

        let delay = self.engine.get_delay(prev_edge_id, edge.edge_id)?;

        state_model.set_time_by_index(state, self.edge_turn_delay_idx, &delay)?;
        state_model.add_time_by_index(state, self.edge_time_idx, &delay)?;

        if let Some(trip_time_idx) = self.trip_time_idx {
            state_model.add_time_by_index(state, trip_time_idx, &delay)?;
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
