use crate::model::network::vertex_id::VertexId;
use std::collections::HashMap;

pub mod a_star;
pub mod backtrack;
mod direction;
mod edge_traversal;
pub mod ksp;
mod search_algorithm;
mod search_algorithm_result;
mod search_error;
mod search_instance;
mod search_result;
mod search_tree_branch;
pub mod util;

pub use direction::Direction;
pub use edge_traversal::EdgeTraversal;
pub use search_algorithm::SearchAlgorithm;
pub use search_algorithm_result::SearchAlgorithmResult;
pub use search_error::SearchError;
pub use search_instance::SearchInstance;
pub use search_result::SearchResult;
pub use search_tree_branch::SearchTreeBranch;

pub type MinSearchTree = HashMap<VertexId, SearchTreeBranch>;
