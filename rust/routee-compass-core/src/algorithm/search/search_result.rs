use super::search_tree_branch::SearchTreeBranch;
use crate::model::network::vertex_id::VertexId;
use std::collections::HashMap;

#[derive(Default)]
pub struct SearchResult {
    pub tree: HashMap<VertexId, SearchTreeBranch>,
    pub iterations: u64,
}

impl SearchResult {
    pub fn new(tree: HashMap<VertexId, SearchTreeBranch>, iterations: u64) -> SearchResult {
        SearchResult { tree, iterations }
    }
}
