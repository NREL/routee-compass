use self::search_tree_branch::SearchTreeBranch;
use crate::model::road_network::vertex_id::VertexId;
use std::collections::HashMap;

pub mod a_star;
pub mod backtrack;
pub mod direction;
pub mod edge_traversal;
pub mod ksp;
pub mod search_algorithm;
pub mod search_algorithm_result;
pub mod search_error;
pub mod search_instance;
pub mod search_orientation;
pub mod search_result;
pub mod search_tree_branch;
pub mod util;

pub type MinSearchTree = HashMap<VertexId, SearchTreeBranch>;
