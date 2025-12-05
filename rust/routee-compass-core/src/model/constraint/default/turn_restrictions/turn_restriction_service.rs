use super::turn_restriction_model::TurnRestrictionConstraintModel;
use crate::model::{
    constraint::{ConstraintModel, ConstraintModelError, ConstraintModelService},
    network::EdgeId,
    state::StateModel,
};
use serde::Deserialize;
use std::{collections::HashSet, sync::Arc};

#[derive(Eq, PartialEq, Hash, Deserialize, Clone)]
pub struct RestrictedEdgePair {
    pub prev_edge_id: EdgeId,
    pub next_edge_id: EdgeId,
}

#[derive(Clone)]
pub struct TurnRestrictionFrontierService {
    pub restricted_edge_pairs: Arc<HashSet<RestrictedEdgePair>>,
}

impl ConstraintModelService for TurnRestrictionFrontierService {
    fn build(
        &self,
        _query: &serde_json::Value,
        _state_model: Arc<StateModel>,
    ) -> Result<Arc<dyn ConstraintModel>, ConstraintModelError> {
        let service: Arc<TurnRestrictionFrontierService> = Arc::new(self.clone());
        let model = TurnRestrictionConstraintModel { service };
        Ok(Arc::new(model))
    }
}
