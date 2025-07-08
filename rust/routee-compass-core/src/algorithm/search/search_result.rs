use crate::model::label::Label;

use super::search_tree_branch::SearchTreeBranch;
use std::collections::HashMap;

#[derive(Default)]
pub struct SearchResult {
    pub tree: HashMap<Label, SearchTreeBranch>,
    pub iterations: u64,
}

impl SearchResult {
    pub fn new(tree: HashMap<Label, SearchTreeBranch>, iterations: u64) -> SearchResult {
        SearchResult { tree, iterations }
    }
}
