use allocative::Allocative;

use routee_compass_core::{
    algorithm::search::{EdgeTraversal, SearchTreeBranch},
    model::label::Label,
};

use std::{collections::HashMap, time::Duration};

#[derive(Allocative)]
pub struct SearchAppResult {
    pub routes: Vec<Vec<EdgeTraversal>>,
    pub trees: Vec<HashMap<Label, SearchTreeBranch>>,
    pub search_executed_time: String,
    pub search_runtime: Duration,
    pub iterations: u64,
}
