use routee_compass_core::model::{
    frontier::{
        frontier_model::FrontierModel, frontier_model_error::FrontierModelError,
        frontier_model_service::FrontierModelService,
    },
    road_network::edge_id::EdgeId,
};
use std::{collections::HashSet, sync::Arc};

use super::turn_restriction_model::TurnRestrictionFrontierModel;

#[derive(Clone)]
pub struct TurnRestrictionFrontierService {
    pub restricted_edges: Arc<HashSet<(EdgeId, EdgeId)>>,
}

impl FrontierModelService for TurnRestrictionFrontierService {
    fn build(
        &self,
        _query: &serde_json::Value,
    ) -> Result<Arc<dyn FrontierModel>, FrontierModelError> {
        let service: Arc<TurnRestrictionFrontierService> = Arc::new(self.clone());
        let model = TurnRestrictionFrontierModel { service };
        Ok(Arc::new(model))
    }
}
