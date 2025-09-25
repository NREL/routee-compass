use crate::algorithm::search::SearchTree;
use super::edge_traversal::EdgeTraversal;

#[derive(Default)]
pub struct SearchAlgorithmResult {
    pub trees: Vec<SearchTree>,
    pub routes: Vec<Vec<EdgeTraversal>>,
    pub iterations: u64,
}
