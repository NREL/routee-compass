use crate::model::unit::{as_f64::AsF64, Cost};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Clone, Copy)]
#[serde(rename_all = "snake_case")]
pub enum CostAggregation {
    Sum,
    Mul,
}

impl CostAggregation {
    pub fn agg(&self, costs: &[(&String, Cost)]) -> Cost {
        match self {
            CostAggregation::Sum => costs.iter().fold(Cost::ZERO, |acc, (_, c)| acc + *c),
            CostAggregation::Mul => {
                if costs.is_empty() {
                    Cost::ZERO
                } else {
                    costs.iter().fold(Cost::ONE, |acc, (_, c)| {
                        Cost::new(acc.as_f64() * c.as_f64())
                    })
                }
            }
        }
    }
}
