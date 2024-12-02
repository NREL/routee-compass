use super::turn_delay_access_model_engine::TurnDelayAccessModelEngine;
use crate::model::{
    access::{access_model::AccessModel, access_model_error::AccessModelError},
    network::{Edge, Vertex},
    state::{state_feature::StateFeature, state_model::StateModel},
    traversal::state::state_variable::StateVar,
};
use std::sync::Arc;

pub struct TurnDelayAccessModel {
    pub engine: Arc<TurnDelayAccessModelEngine>,
}

impl AccessModel for TurnDelayAccessModel {
    fn access_edge(
        &self,
        traversal: (&Vertex, &Edge, &Vertex, &Edge, &Vertex),
        state: &mut Vec<StateVar>,
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
