use super::turn_delay_access_model_engine::TurnDelayAccessModelEngine;
use crate::model::{
    access::{AccessModel, AccessModelError},
    network::{Edge, Vertex},
    state::{StateFeature, StateModel, StateVariable},
};
use std::sync::Arc;

pub struct TurnDelayAccessModel {
    pub engine: Arc<TurnDelayAccessModelEngine>,
}

impl TurnDelayAccessModel {
    pub const EDGE_TIME: &'static str = "edge_time";
    pub const TRIP_TIME: &'static str = "trip_time";

    pub fn new(engine: Arc<TurnDelayAccessModelEngine>) -> Self {
        TurnDelayAccessModel { engine }
    }
}

impl AccessModel for TurnDelayAccessModel {
    fn access_edge(
        &self,
        traversal: (&Vertex, &Edge, &Vertex, &Edge, &Vertex),
        state: &mut Vec<StateVariable>,
        state_model: &StateModel,
    ) -> Result<(), AccessModelError> {
        let delay = self.engine.get_delay(traversal)?;
        state_model.set_time(state, Self::EDGE_TIME, &delay)?;
        state_model.add_time(state, Self::TRIP_TIME, &delay)?;
        Ok(())
    }

    fn state_features(&self) -> Vec<(String, StateFeature)> {
        vec![]
    }
}
