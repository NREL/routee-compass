use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GraphConfig {
    pub vertex_list_input_file: String,
    pub edge_list: Vec<EdgeListConfig>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EdgeListConfig {
    pub edge_list_input_file: String,
}