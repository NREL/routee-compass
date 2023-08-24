use std::collections::HashSet;

use crate::model::{
    frontier::{frontier_model::FrontierModel, frontier_model_error::FrontierModelError},
    property::{edge::Edge, road_class::RoadClass},
    traversal::state::traversal_state::TraversalState,
};

pub struct RoadClassFrontierModel {
    pub valid_road_classes: HashSet<RoadClass>,
}

impl FrontierModel for RoadClassFrontierModel {
    fn valid_frontier(
        &self,
        edge: &Edge,
        _state: &TraversalState,
    ) -> Result<bool, FrontierModelError> {
        let road_class = edge.road_class;
        Ok(self.valid_road_classes.contains(&road_class))
    }
}
