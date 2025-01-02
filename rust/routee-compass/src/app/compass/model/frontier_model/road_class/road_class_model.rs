use super::road_class_service::RoadClassFrontierService;
use routee_compass_core::{
    algorithm::search::SearchTreeBranch,
    model::{
        frontier::{FrontierModel, FrontierModelError},
        network::{Edge, VertexId},
        state::{StateModel, StateVariable},
    },
};
use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

pub struct RoadClassFrontierModel {
    pub service: Arc<RoadClassFrontierService>,
    pub road_classes: Option<HashSet<u8>>,
}

impl FrontierModel for RoadClassFrontierModel {
    fn valid_frontier(
        &self,
        edge: &Edge,
        _state: &[StateVariable],
        _tree: &HashMap<VertexId, SearchTreeBranch>,
        _direction: &routee_compass_core::algorithm::search::Direction,
        _state_model: &StateModel,
    ) -> Result<bool, FrontierModelError> {
        self.valid_edge(edge)
    }

    fn valid_edge(&self, edge: &Edge) -> Result<bool, FrontierModelError> {
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
