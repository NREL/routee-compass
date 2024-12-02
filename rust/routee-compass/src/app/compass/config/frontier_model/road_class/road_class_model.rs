use super::road_class_service::RoadClassFrontierService;
use routee_compass_core::model::{
    frontier::{frontier_model::FrontierModel, frontier_model_error::FrontierModelError},
    network::Edge,
    state::state_model::StateModel,
    traversal::state::state_variable::StateVar,
};
use std::{collections::HashSet, sync::Arc};

pub struct RoadClassFrontierModel {
    pub service: Arc<RoadClassFrontierService>,
    pub road_classes: Option<HashSet<u8>>,
}

impl FrontierModel for RoadClassFrontierModel {
    fn valid_frontier(
        &self,
        edge: &Edge,
        _state: &[StateVar],
        _previous_edge: Option<&Edge>,
        _state_model: &StateModel,
    ) -> Result<bool, FrontierModelError> {
        match &self.road_classes {
            None => Ok(true),
            Some(road_classes) => self
                .service
                .road_class_lookup
                .get(edge.edge_id.0)
                .ok_or_else(|| {
                    FrontierModelError::FrontierModelError(format!(
                        "edge id {} missing from frontier model file",
                        edge.edge_id
                    ))
                })
                .map(|road_class| road_classes.contains(road_class)),
        }
    }
}
