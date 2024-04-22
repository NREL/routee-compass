use std::collections::{HashMap, HashSet};

use crate::{
    algorithm::search::{
        edge_traversal::EdgeTraversal, search_error::SearchError, search_instance::SearchInstance,
    },
    model::unit::as_f64::AsF64,
};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum RouteSimilarityFunction {
    EdgeIdCosineSimilarity { threshold: f64 },
    DistanceWeightedCosineSimilarity { threshold: f64 },
}

impl RouteSimilarityFunction {
    /// tests if a similarity rank value is dissimilar enough.
    ///
    /// # Arguments
    /// * `similarity` - output of this rank_similarity function
    ///
    /// # Result
    ///
    /// true if the rank is sufficiently dissimilar
    pub fn sufficiently_dissimilar(&self, similarity: f64) -> bool {
        match self {
            RouteSimilarityFunction::EdgeIdCosineSimilarity { threshold } => {
                similarity <= *threshold
            }
            RouteSimilarityFunction::DistanceWeightedCosineSimilarity { threshold } => {
                similarity <= *threshold
            }
        }
    }

    /// tests if a similarity rank value is similar enough.
    ///
    /// # Arguments
    /// * `similarity` - output of this rank_similarity function
    ///
    /// # Result
    ///
    /// true if the rank is sufficiently similar
    pub fn sufficiently_similar(&self, similarity: f64) -> bool {
        !self.sufficiently_dissimilar(similarity)
    }

    /// ranks the similarity of two routes using this similarity function.
    ///
    /// # Arguments
    /// * `a`  - one route
    /// * `b`  - another route
    /// * `si` - search instance for these routes
    ///
    /// # Returns
    /// the similarity ranking of these routes
    pub fn rank_similarity(
        &self,
        a: &[EdgeTraversal],
        b: &[EdgeTraversal],
        si: &SearchInstance,
    ) -> Result<f64, SearchError> {
        match self {
            RouteSimilarityFunction::EdgeIdCosineSimilarity { threshold: _ } => {
                let numer = a
                    .iter()
                    .map(|e| e.edge_id)
                    .collect::<HashSet<_>>()
                    .intersection(&b.iter().map(|e| e.edge_id).collect::<HashSet<_>>())
                    .collect::<Vec<_>>()
                    .len();
                let denom = a.len() + b.len();
                if denom == 0 {
                    Ok(0.0)
                } else {
                    Ok(numer as f64 / denom as f64)
                }
            }
            RouteSimilarityFunction::DistanceWeightedCosineSimilarity { threshold: _ } => {
                let a_set = a
                    .iter()
                    .map(|e| {
                        si.directed_graph
                            .get_edge(e.edge_id)
                            .map(|edge| (e.edge_id, edge.distance.as_f64()))
                    })
                    .collect::<Result<HashMap<_, _>, _>>()?;
                let b_set = b
                    .iter()
                    .flat_map(|e| a_set.get(&e.edge_id).copied())
                    .collect::<Vec<_>>();
                let numer: f64 = b_set.into_iter().sum();
                let denom = a.len() + b.len();
                if denom == 0 {
                    Ok(0.0)
                } else {
                    Ok(numer / denom as f64)
                }
            }
        }
    }
}
