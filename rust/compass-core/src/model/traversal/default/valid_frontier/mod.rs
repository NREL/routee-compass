use crate::model::{
    property::edge::Edge,
    traversal::{
        state::traversal_state::TraversalState, traversal_model_error::TraversalModelError,
    },
};

pub mod road_class;

pub type ValidFunction =
    Box<dyn Fn(&Edge, &TraversalState) -> Result<bool, TraversalModelError> + Send + Sync>;
