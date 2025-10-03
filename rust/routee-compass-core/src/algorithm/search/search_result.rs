use crate::algorithm::search::SearchTree;

#[derive(Default)]
pub struct SearchResult {
    pub tree: SearchTree,
    pub iterations: u64,
}

impl SearchResult {
    pub fn new(tree: SearchTree, iterations: u64) -> SearchResult {
        SearchResult { tree, iterations }
    }
}
