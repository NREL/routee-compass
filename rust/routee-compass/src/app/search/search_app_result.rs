use crate::plugin::{
    output::default::traversal::traversal_output_format::TraversalOutputFormat,
    plugin_error::PluginError,
};
use allocative::Allocative;
use geo::LineString;
use routee_compass_core::{
    algorithm::search::{edge_traversal::EdgeTraversal, search_tree_branch::SearchTreeBranch},
    model::road_network::vertex_id::VertexId,
};
use serde_json::json;
use std::{collections::HashMap, time::Duration};

#[derive(Allocative)]
pub struct SearchAppResult {
    pub routes: Vec<Vec<EdgeTraversal>>,
    pub trees: Vec<HashMap<VertexId, SearchTreeBranch>>,
    pub search_executed_time: String,
    pub search_runtime: Duration,
    pub iterations: u64,
}
