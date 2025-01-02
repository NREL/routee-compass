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

impl AccessModel for TurnDelayAccessModel {
    fn access_edge(
        &self,
        traversal: (&Vertex, &Edge, &Vertex, &Edge, &Vertex),
        state: &mut Vec<StateVariable>,
        state_model: &StateModel,
    ) -> Result<(), AccessModelError> {
        let (delay, delay_unit) = self.engine.get_delay(traversal)?;
        state_model.add_time(state, &self.engine.time_feature_name, &delay, delay_unit)?;
        Ok(())
    }

    fn state_features(&self) -> Vec<(String, StateFeature)> {
        vec![]
    }
}
