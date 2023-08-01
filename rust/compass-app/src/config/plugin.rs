use serde::Deserialize;


#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
pub enum InputPluginConfig {
    #[serde(rename = "vertex_rtree")]
    VertexRTree,
}


#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
pub enum OutputPluginConfig {
    #[serde(rename = "geometry")]
    Geometry { edge_file: String },
}

#[derive(Debug, Deserialize)]
pub struct PluginConfig {
    input_plugins: Vec<InputPluginConfig>,
    output_plugins: Vec<OutputPluginConfig>,
}

