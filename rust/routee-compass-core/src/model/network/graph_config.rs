use serde::{Deserialize, Serialize};

use crate::config::OneOrMany;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GraphConfig {
    pub vertex_list_input_file: String,
    pub edge_list: OneOrMany<EdgeListConfig>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EdgeListConfig {
    pub input_file: String,
}
