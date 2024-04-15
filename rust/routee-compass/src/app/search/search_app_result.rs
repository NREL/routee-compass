
use allocative::Allocative;

use routee_compass_core::{
    algorithm::search::{edge_traversal::EdgeTraversal, search_tree_branch::SearchTreeBranch},
    model::road_network::vertex_id::VertexId,
};

use std::{collections::HashMap, time::Duration};

#[derive(Allocative)]
pub struct SearchAppResult {
    pub routes: Vec<Vec<EdgeTraversal>>,
    pub trees: Vec<HashMap<VertexId, SearchTreeBranch>>,
    pub search_executed_time: String,
    pub search_runtime: Duration,
    pub iterations: u64,
}
