use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub enum GraphConfig {
    #[serde(rename = "tomtom")]
    TomTom {
        edge_file: String,
        vertex_file: String,
        n_edges: Option<usize>,
        n_vertices: Option<usize>,
        verbose: bool,
    },
}
