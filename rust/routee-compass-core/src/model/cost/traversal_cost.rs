use std::collections::HashMap;

use allocative::Allocative;
use serde::{Deserialize, Serialize};

use crate::model::unit::Cost;

/// the cost of an edge traversal.
#[derive(Serialize, Deserialize, Default, Clone, Debug, Allocative)]
pub struct TraversalCost {
    pub total_cost: Cost,
    pub components: HashMap<String, Cost>,
}

impl TraversalCost {
    /// inserts a new cost. if it already exists, we append to the cost value.
    pub fn insert(&mut self, name: &str, cost: Cost) {
        self.total_cost += cost;
        self.components
            .entry(name.to_string())
            .and_modify(|v| *v += cost)
            .or_insert(cost);
    }
}
