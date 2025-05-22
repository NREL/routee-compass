use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FeatureBounds {
    pub lower_bound: f64,
    pub upper_bound: f64,
    pub num_bins: usize,
}
