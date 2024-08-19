use std::collections::{HashMap, HashSet};

use crate::{
    algorithm::search::{
        edge_traversal::EdgeTraversal, search_error::SearchError, search_instance::SearchInstance,
    },
    model::{road_network::edge_id::EdgeId, unit::as_f64::AsF64},
};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Default, Clone)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum RouteSimilarityFunction {
    #[default]
    AcceptAll,
    EdgeIdCosineSimilarity {
        threshold: f64,
    },
    DistanceWeightedCosineSimilarity {
        threshold: f64,
    },
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
            RouteSimilarityFunction::AcceptAll => true,
            RouteSimilarityFunction::EdgeIdCosineSimilarity { threshold } => {
                similarity <= *threshold
            }
            RouteSimilarityFunction::DistanceWeightedCosineSimilarity { threshold } => {
                similarity <= *threshold
            }
        }
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
            RouteSimilarityFunction::AcceptAll => Ok(0.0),
            RouteSimilarityFunction::EdgeIdCosineSimilarity { threshold: _ } => {
                let unit_dist_fn = Box::new(|_| Ok(1.0));
                cos_similarity(a, b, unit_dist_fn)
            }
            RouteSimilarityFunction::DistanceWeightedCosineSimilarity { threshold: _ } => {
                let dist_fn = Box::new(|edge_id| {
                    si.directed_graph
                        .get_edge(edge_id)
                        .map(|edge| edge.distance.as_f64())
                        .map_err(SearchError::GraphError)
                });
                cos_similarity(a, b, dist_fn)
            }
        }
    }
}

/// computes the cosine similarity of two routes using the provided distance function
///
/// # Arguments
///
/// * `a`       - this route
/// * `b`       - that route
/// * `dist_fn` - mapping from EdgeIds to some distance value
///
/// # Returns
///
/// the cosine similarity of the routes, a value from -1 to 1
fn cos_similarity<'a>(
    a: &[EdgeTraversal],
    b: &[EdgeTraversal],
    dist_fn: Box<dyn Fn(EdgeId) -> Result<f64, SearchError> + 'a>,
) -> Result<f64, SearchError> {
    let a_map = a
        .iter()
        .map(|e| dist_fn(e.edge_id).map(|dist| (e.edge_id, dist)))
        .collect::<Result<HashMap<_, _>, _>>()?;
    let b_map = b
        .iter()
        .map(|e| dist_fn(e.edge_id).map(|dist| (e.edge_id, dist)))
        .collect::<Result<HashMap<_, _>, _>>()?;

    let numer: f64 = a_map
        .keys()
        .collect::<HashSet<_>>()
        .union(&b_map.keys().collect::<HashSet<_>>())
        .map(|edge_id| {
            let a_dist = a_map.get(edge_id).cloned().unwrap_or_default();
            let b_dist = b_map.get(edge_id).cloned().unwrap_or_default();
            a_dist * b_dist
        })
        .sum();

    let denom_a: f64 = a_map.values().map(|d| d * d).sum();
    let denom_b: f64 = b_map.values().map(|d| d * d).sum();
    let denom = denom_a.sqrt() * denom_b.sqrt();
    let cos_sim = numer / denom;
    Ok(cos_sim)
}
