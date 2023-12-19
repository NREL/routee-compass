use crate::util::unit::as_f64::AsF64;

use super::cost::Cost;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Clone, Copy)]
#[serde(rename_all = "snake_case")]
pub enum CostAggregation {
    Sum,
    Mul,
}

impl CostAggregation {
    pub fn agg(&self, costs: &[Cost]) -> Cost {
        match self {
            CostAggregation::Sum => costs.iter().fold(Cost::ZERO, |acc, c| acc + *c),
            CostAggregation::Mul => {
                if costs.len() == 0 {
                    Cost::ZERO
                } else {
                    costs
                        .iter()
                        .fold(Cost::ONE, |acc, c| Cost::new(acc.as_f64() * c.as_f64()))
                }
            }
        }
    }
}
