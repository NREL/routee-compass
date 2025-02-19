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
    pub query_road_classes: Option<HashSet<String>>,
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
        match &self.query_road_classes {
            None => Ok(true),
            Some(road_classes) => self
                .service
                .road_class_by_edge
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

#[cfg(test)]
mod test {
    use crate::app::compass::model::frontier_model::road_class::road_class_service::RoadClassFrontierService;
    use routee_compass_core::model::{
        frontier::{FrontierModel, FrontierModelService},
        network::Edge,
        state::StateModel,
    };
    use serde_json::{json, Value};
    use std::sync::Arc;

    /// builds the test model for a given RoadClassModel test
    /// # Arguments
    /// * `road_class_vector` - the value assumed to be read from a file, with road classes by EdgeId index value
    /// * `query` - the user query which should provide the set of valid road classes for this search
    fn mock(road_class_vector: Box<[String]>, query: Value) -> Arc<dyn FrontierModel> {
        let service = Arc::new(RoadClassFrontierService {
            road_class_by_edge: Arc::new(road_class_vector),
        });
        let state_model = Arc::new(StateModel::empty());
        service.build(&query, state_model.clone()).unwrap()
    }

    #[test]
    fn test_no_road_classes() {
        let model = mock(Box::new([String::from("a")]), json!({}));
        let result = model.valid_edge(&Edge::new(0, 0, 1, 1.0)).unwrap();
        assert!(result)
    }

    #[test]
    fn test_valid_class() {
        let model = mock(
            Box::new([String::from("a")]),
            json!({"road_classes": ["a"]}),
        );
        let result = model.valid_edge(&Edge::new(0, 0, 1, 1.0)).unwrap();
        assert!(result)
    }

    #[test]
    fn test_invalid_class() {
        let model = mock(
            Box::new([String::from("oh no!")]),
            json!({"road_classes": ["a"]}),
        );
        let result = model.valid_edge(&Edge::new(0, 0, 1, 1.0)).unwrap();
        assert!(!result)
    }

    #[test]
    fn test_one_of_valid_classes() {
        let model = mock(
            Box::new([String::from("a")]),
            json!({"road_classes": ["a", "b", "c"]}),
        );
        let result = model.valid_edge(&Edge::new(0, 0, 1, 1.0)).unwrap();
        assert!(result)
    }

    #[test]
    fn test_none_of_valid_classes() {
        let model = mock(
            Box::new([String::from("oh no!")]),
            json!({"road_classes": ["a", "b", "c"]}),
        );
        let result = model.valid_edge(&Edge::new(0, 0, 1, 1.0)).unwrap();
        assert!(!result)
    }

    #[test]
    fn test_valid_numeric_class() {
        let model = mock(Box::new([String::from("1")]), json!({"road_classes": [1]}));
        let result = model.valid_edge(&Edge::new(0, 0, 1, 1.0)).unwrap();
        assert!(result)
    }

    #[test]
    fn test_invalid_numeric_class() {
        let model = mock(
            Box::new([String::from("OH NO!")]),
            json!({"road_classes": [1]}),
        );
        let result = model.valid_edge(&Edge::new(0, 0, 1, 1.0)).unwrap();
        assert!(!result)
    }

    #[test]
    fn test_valid_boolean_class() {
        let model = mock(
            Box::new([String::from("true")]),
            json!({"road_classes": [true]}),
        );
        let result = model.valid_edge(&Edge::new(0, 0, 1, 1.0)).unwrap();
        assert!(result)
    }

    #[test]
    fn test_invalid_boolean_class() {
        let model = mock(
            Box::new([String::from("OH NO!")]),
            json!({"road_classes": [true]}),
        );
        let result = model.valid_edge(&Edge::new(0, 0, 1, 1.0)).unwrap();
        assert!(!result)
    }
}
