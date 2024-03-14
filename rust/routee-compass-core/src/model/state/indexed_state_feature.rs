use std::fmt::Display;

use super::state_feature::StateFeature;
use serde::{Deserialize, Serialize};

/// simple record type which couples the state variable index with the feature
/// representation for a given state variable in the StateModel.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct IndexedStateFeature {
    pub index: usize,
    pub feature: StateFeature,
}

impl Display for IndexedStateFeature {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let feature_str = serde_json::to_string(&self.feature).unwrap_or(String::from("<err!>"));
        let string = format!("{}: {}", self.index, feature_str);
        f.write_str(&string)
    }
}

impl IndexedStateFeature {
    pub fn new(index: usize, feature: StateFeature) -> IndexedStateFeature {
        IndexedStateFeature { index, feature }
    }
}
