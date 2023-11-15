use self::search_tree_branch::SearchTreeBranch;
use crate::model::road_network::vertex_id::VertexId;
use std::collections::HashMap;

pub mod a_star;
pub mod backtrack;
pub mod direction;
pub mod edge_traversal;
pub mod search_algorithm;
pub mod search_algorithm_type;
pub mod search_error;
pub mod search_tree_branch;

pub type MinSearchTree = HashMap<VertexId, SearchTreeBranch>;
