use std::sync::Arc;

use crate::model::{
    frontier::{
        frontier_model::FrontierModel, frontier_model_error::FrontierModelError,
        frontier_model_service::FrontierModelService,
    },
    property::edge::Edge,
    traversal::state::traversal_state::TraversalState,
};

#[derive(Clone)]
pub struct RoadClassFrontierService {
    pub road_class_lookup: Arc<Vec<bool>>,
}

pub struct RoadClassFrontier {
    pub service: Arc<RoadClassFrontierService>,
}

impl FrontierModel for RoadClassFrontier {
    fn valid_frontier(
        &self,
        edge: &Edge,
        _state: &TraversalState,
    ) -> Result<bool, FrontierModelError> {
        self.service
            .road_class_lookup
            .get(edge.edge_id.0)
            .ok_or(FrontierModelError::MissingIndex(format!(
                "{}",
                edge.edge_id
            )))
            .cloned()
    }
}

impl FrontierModelService for RoadClassFrontierService {
    fn build(
        &self,
        _query: &serde_json::Value,
    ) -> Result<Arc<dyn FrontierModel>, FrontierModelError> {
        let service: Arc<RoadClassFrontierService> = Arc::new(self.clone());
        let model = RoadClassFrontier { service };
        Ok(Arc::new(model))
    }
}
