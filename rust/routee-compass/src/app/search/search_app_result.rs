use allocative::Allocative;

use routee_compass_core::algorithm::search::{EdgeTraversal, SearchTree};

use std::time::Duration;

#[derive(Allocative)]
pub struct SearchAppResult {
    pub routes: Vec<Vec<EdgeTraversal>>,
    pub trees: Vec<SearchTree>,
    pub search_executed_time: String,
    pub search_runtime: Duration,
    pub iterations: u64,
    pub terminated: Option<String>,
}
