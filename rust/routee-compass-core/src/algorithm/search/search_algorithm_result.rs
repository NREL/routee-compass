use crate::model::label::Label;

use super::{edge_traversal::EdgeTraversal, search_tree_branch::SearchTreeBranch};
use std::collections::HashMap;

#[derive(Default)]
pub struct SearchAlgorithmResult {
    pub trees: Vec<HashMap<Label, SearchTreeBranch>>,
    pub routes: Vec<Vec<EdgeTraversal>>,
    pub iterations: u64,
}
