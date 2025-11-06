use super::edge_traversal::EdgeTraversal;
use crate::algorithm::search::SearchTree;

#[derive(Default)]
pub struct SearchAlgorithmResult {
    pub trees: Vec<SearchTree>,
    pub routes: Vec<Vec<EdgeTraversal>>,
    pub iterations: u64,
    pub terminated: Option<String>,
}
