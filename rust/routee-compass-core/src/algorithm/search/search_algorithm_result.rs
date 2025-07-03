use super::{edge_traversal::EdgeTraversal, search_tree_branch::SearchTreeBranch};
use crate::model::label::label_model::Label;
use std::collections::HashMap;

#[derive(Default)]
pub struct SearchAlgorithmResult {
    pub trees: Vec<HashMap<Label, SearchTreeBranch>>,
    pub routes: Vec<Vec<EdgeTraversal>>,
    pub iterations: u64,
}
