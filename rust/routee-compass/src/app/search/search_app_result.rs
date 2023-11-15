use chrono::{DateTime, Local};
use routee_compass_core::{
    algorithm::search::{edge_traversal::EdgeTraversal, search_tree_branch::SearchTreeBranch},
    model::road_network::vertex_id::VertexId,
};
use std::{collections::HashMap, time::Duration};

pub struct SearchAppResult {
    pub route: Vec<EdgeTraversal>,
    pub tree: HashMap<VertexId, SearchTreeBranch>,
    pub search_start_time: DateTime<Local>,
    pub search_runtime: Duration,
    pub route_runtime: Duration,
    pub total_runtime: Duration,
}
