use std::collections::HashSet;

use crate::model::{
    property::{edge::Edge, road_class::RoadClass},
    traversal::state::traversal_state::TraversalState,
};

use super::ValidFunction;

/// Builds a valid frontier function that checks if the road class of the edge is in the given set.
pub fn build_road_class_valid_fn(valid_road_classes: HashSet<RoadClass>) -> ValidFunction {
    Box::new(move |edge: &Edge, _state: &TraversalState| {
        let road_class = edge.road_class;
        Ok(valid_road_classes.contains(&road_class))
    })
}
