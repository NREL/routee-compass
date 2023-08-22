use crate::model::{
    property::{edge::Edge, road_class::RoadClass},
    traversal::state::traversal_state::TraversalState,
};

use super::ValidFunction;

pub fn build_road_class_valid_fn(max_road_class: RoadClass) -> ValidFunction {
    Box::new(move |edge: &Edge, _state: &TraversalState| {
        let road_class = edge.road_class;
        Ok(road_class <= max_road_class)
    })
}
