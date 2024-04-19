use serde::{Deserialize, Serialize};

use crate::algorithm::search::edge_traversal::EdgeTraversal;

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum RouteSimilarityFunction {}

impl RouteSimilarityFunction {
    pub fn rank_similarity(&self, a: Vec<EdgeTraversal>, b: Vec<EdgeTraversal>) -> f64 {
        todo!()
    }
}
