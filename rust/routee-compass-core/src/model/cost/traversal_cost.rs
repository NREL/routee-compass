use std::collections::HashMap;

use allocative::Allocative;
use serde::{Deserialize, Serialize};

use crate::model::unit::Cost;

/// the cost of an edge traversal.
#[derive(Serialize, Deserialize, Default, Clone, Debug, Allocative)]
pub struct TraversalCost {
    /// the cost components with user-defined weighting objectives applied
    pub objective_cost: Cost,
    /// the true total cost of this traversal
    pub total_cost: Cost,
    /// breakdown of the components of the cost
    pub cost_component: HashMap<String, Cost>,
}

impl TraversalCost {
    /// inserts a new cost into this traversal.
    /// manages storing a separate notion of objective vs total cost
    /// by only applying the "weight" value to the objective cost.
    ///
    /// when recording a cost component, if it already exists, we append to the cost value.
    pub fn insert(&mut self, name: &str, cost: Cost, weight: f64) {
        let positive_cost = Cost::enforce_strictly_positive(cost);
        self.total_cost += positive_cost;
        self.objective_cost += positive_cost * weight;
        self.cost_component
            .entry(name.to_string())
            .and_modify(|v| *v += positive_cost)
            .or_insert(positive_cost);
    }
}
