use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub enum GraphConfig {
    #[serde(rename = "tomtom")]
    TomTom {
        edge_file: String,
        vertex_file: String,
    },
}
