use super::turn_restriction_model::TurnRestrictionFrontierModel;
use routee_compass_core::model::{
    frontier::{FrontierModel, FrontierModelError, FrontierModelService},
    network::edge_id::EdgeId,
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

impl FrontierModelService for TurnRestrictionFrontierService {
    fn build(
        &self,
        _query: &serde_json::Value,
        _state_model: Arc<StateModel>,
    ) -> Result<Arc<dyn FrontierModel>, FrontierModelError> {
        let service: Arc<TurnRestrictionFrontierService> = Arc::new(self.clone());
        let model = TurnRestrictionFrontierModel { service };
        Ok(Arc::new(model))
    }
}
