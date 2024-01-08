use super::road_class_service::RoadClassFrontierService;
use routee_compass_core::model::{
    frontier::{frontier_model::FrontierModel, frontier_model_error::FrontierModelError},
    property::edge::Edge,
    traversal::state::traversal_state::TraversalState,
};
use std::{collections::HashSet, sync::Arc};

pub struct RoadClassFrontierModel {
    pub service: Arc<RoadClassFrontierService>,
    pub road_classes: Option<HashSet<String>>,
}

impl FrontierModel for RoadClassFrontierModel {
    fn valid_frontier(
        &self,
        edge: &Edge,
        _state: &TraversalState,
        _previous_edge: Option<&Edge>,
    ) -> Result<bool, FrontierModelError> {
        match &self.road_classes {
            None => Ok(true),
            Some(road_classes) => self
                .service
                .road_class_lookup
                .get(edge.edge_id.0)
                .ok_or_else(|| FrontierModelError::MissingIndex(format!("{}", edge.edge_id)))
                .map(|road_class| road_classes.contains(road_class)),
        }
    }
}
