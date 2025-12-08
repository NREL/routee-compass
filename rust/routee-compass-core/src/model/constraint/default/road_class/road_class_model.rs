use super::road_class_service::RoadClassFrontierService;
use crate::model::{
    constraint::{ConstraintModel, ConstraintModelError},
    network::Edge,
    state::{StateModel, StateVariable},
};
use std::{collections::HashSet, sync::Arc};

pub struct RoadClassConstraintModel {
    pub service: Arc<RoadClassFrontierService>,
    pub query_road_classes: Option<HashSet<String>>,
}

impl ConstraintModel for RoadClassConstraintModel {
    fn valid_frontier(
        &self,
        edge: &Edge,
        _previous_edge: Option<&Edge>,
        _state: &[StateVariable],
        _state_model: &StateModel,
    ) -> Result<bool, ConstraintModelError> {
        self.valid_edge(edge)
    }

    fn valid_edge(&self, edge: &Edge) -> Result<bool, ConstraintModelError> {
        match &self.query_road_classes {
            None => Ok(true),
            Some(road_classes) => self
                .service
                .road_class_by_edge
                .get(edge.edge_id.0)
                .ok_or_else(|| {
                    ConstraintModelError::ConstraintModelError(format!(
                        "edge id {} missing from constraint model file",
                        edge.edge_id
                    ))
                })
                .map(|road_class| road_classes.contains(road_class)),
        }
    }
}

#[cfg(test)]
mod test {
    use crate::model::{
        constraint::{
            default::road_class::road_class_service::RoadClassFrontierService, ConstraintModel,
            ConstraintModelService,
        },
        network::Edge,
        state::StateModel,
    };
    use serde_json::{json, Value};
    use std::sync::Arc;
    use uom::si::f64::Length;

    /// builds the test model for a given RoadClassModel test
    /// # Arguments
    /// * `road_class_vector` - the value assumed to be read from a file, with road classes by EdgeId index value
    /// * `query` - the user query which should provide the set of valid road classes for this search
    fn mock(road_class_vector: Box<[String]>, query: Value) -> Arc<dyn ConstraintModel> {
        let service = Arc::new(RoadClassFrontierService {
            road_class_by_edge: Arc::new(road_class_vector),
        });
        let state_model = Arc::new(StateModel::empty());
        service.build(&query, state_model.clone()).unwrap()
    }

    fn mock_edge() -> Edge {
        Edge::new(0, 0, 0, 1, Length::new::<uom::si::length::meter>(1.0))
    }

    #[test]
    fn test_no_road_classes() {
        let model = mock(Box::new([String::from("a")]), json!({}));
        let edge = mock_edge();
        let result = model.valid_edge(&edge).unwrap();
        assert!(result)
    }

    #[test]
    fn test_valid_class() {
        let model = mock(
            Box::new([String::from("a")]),
            json!({"road_classes": ["a"]}),
        );
        let edge = mock_edge();
        let result = model.valid_edge(&edge).unwrap();
        assert!(result)
    }

    #[test]
    fn test_invalid_class() {
        let model = mock(
            Box::new([String::from("oh no!")]),
            json!({"road_classes": ["a"]}),
        );
        let edge = mock_edge();
        let result = model.valid_edge(&edge).unwrap();
        assert!(!result)
    }

    #[test]
    fn test_one_of_valid_classes() {
        let model = mock(
            Box::new([String::from("a")]),
            json!({"road_classes": ["a", "b", "c"]}),
        );
        let edge = mock_edge();
        let result = model.valid_edge(&edge).unwrap();
        assert!(result)
    }

    #[test]
    fn test_none_of_valid_classes() {
        let model = mock(
            Box::new([String::from("oh no!")]),
            json!({"road_classes": ["a", "b", "c"]}),
        );
        let edge = mock_edge();
        let result = model.valid_edge(&edge).unwrap();
        assert!(!result)
    }

    #[test]
    fn test_valid_numeric_class() {
        let model = mock(Box::new([String::from("1")]), json!({"road_classes": [1]}));
        let edge = mock_edge();
        let result = model.valid_edge(&edge).unwrap();
        assert!(result)
    }

    #[test]
    fn test_invalid_numeric_class() {
        let model = mock(
            Box::new([String::from("OH NO!")]),
            json!({"road_classes": [1]}),
        );
        let edge = mock_edge();
        let result = model.valid_edge(&edge).unwrap();
        assert!(!result)
    }

    #[test]
    fn test_valid_boolean_class() {
        let model = mock(
            Box::new([String::from("true")]),
            json!({"road_classes": [true]}),
        );
        let edge = mock_edge();
        let result = model.valid_edge(&edge).unwrap();
        assert!(result)
    }

    #[test]
    fn test_invalid_boolean_class() {
        let model = mock(
            Box::new([String::from("OH NO!")]),
            json!({"road_classes": [true]}),
        );
        let edge = mock_edge();
        let result = model.valid_edge(&edge).unwrap();
        assert!(!result)
    }
}
