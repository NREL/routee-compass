use crate::model::{
    frontier::{frontier_model::FrontierModel, frontier_model_error::FrontierModelError},
    property::edge::Edge,
    traversal::state::traversal_state::TraversalState,
};

pub struct RoadClassFrontierModel {
    pub road_class_lookup: Vec<bool>,
}

impl FrontierModel for RoadClassFrontierModel {
    fn valid_frontier(
        &self,
        edge: &Edge,
        _state: &TraversalState,
    ) -> Result<bool, FrontierModelError> {
        self.road_class_lookup
            .get(edge.edge_id.0)
            .ok_or(FrontierModelError::MissingIndex(format!(
                "{}",
                edge.edge_id
            )))
            .cloned()
    }
}
