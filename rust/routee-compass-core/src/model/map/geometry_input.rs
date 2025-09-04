use serde::{Deserialize, Serialize};

/// points to a file that contains geometries for one edge list in the graph.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct GeometryInput {
    pub input_file: String,
}
