use std::collections::HashMap;
use std::sync::Arc;

use allocative::Allocative;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::model::unit::Cost;

// Custom serialization/deserialization for HashMap<Arc<str>, Cost>
// Serializes as HashMap<String, Cost> for compatibility
mod arc_str_map {
    use super::*;
    use std::sync::Arc;

    pub fn serialize<S>(map: &HashMap<Arc<str>, Cost>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // Serialize as HashMap<&str, Cost> which serde can handle
        let string_map: HashMap<&str, &Cost> = map.iter().map(|(k, v)| (k.as_ref(), v)).collect();
        string_map.serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<HashMap<Arc<str>, Cost>, D::Error>
    where
        D: Deserializer<'de>,
    {
        // Deserialize as HashMap<String, Cost> then convert to Arc<str>
        let string_map: HashMap<String, Cost> = HashMap::deserialize(deserializer)?;
        Ok(string_map
            .into_iter()
            .map(|(k, v)| (Arc::from(k.as_str()), v))
            .collect())
    }
}

/// the cost of an edge traversal.
#[derive(Serialize, Deserialize, Default, Clone, Debug, Allocative)]
pub struct TraversalCost {
    /// the cost components with user-defined weighting objectives applied
    pub objective_cost: Cost,
    /// the true total cost of this traversal
    pub total_cost: Cost,
    /// breakdown of the components of the cost. uses Arc<str> keys to minimize
    /// memory overhead when millions of TraversalCost instances exist during search.
    #[serde(with = "arc_str_map")]
    pub cost_component: HashMap<Arc<str>, Cost>,
}

impl TraversalCost {
    /// inserts a new cost into this traversal.
    /// manages storing a separate notion of objective vs total cost
    /// by only applying the "weight" value to the objective cost.
    ///
    /// when recording a cost component, if it already exists, we append to the cost value.
    pub fn insert(&mut self, name: Arc<str>, cost: Cost, weight: f64) {
        let positive_cost = Cost::enforce_strictly_positive(cost);
        self.total_cost += positive_cost;
        self.objective_cost += positive_cost * weight;
        self.cost_component
            .entry(name)
            .and_modify(|v| *v += positive_cost)
            .or_insert(positive_cost);
    }
}
