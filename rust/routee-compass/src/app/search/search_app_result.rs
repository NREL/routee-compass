use allocative::Allocative;
use routee_compass_core::{
    algorithm::search::{
        edge_traversal::EdgeTraversal, search_instance::SearchInstance,
        search_tree_branch::SearchTreeBranch,
    },
    model::road_network::vertex_id::VertexId,
};
use std::{collections::HashMap, time::Duration};

#[derive(Allocative)]
pub struct SearchAppResult {
    pub route: Vec<EdgeTraversal>,
    pub tree: HashMap<VertexId, SearchTreeBranch>,
    pub search_executed_time: String,
    pub algorithm_runtime: Duration,
    pub route_runtime: Duration,
    pub search_app_runtime: Duration,
    pub iterations: u64,
}
